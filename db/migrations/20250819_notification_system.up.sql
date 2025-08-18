-- Notification system tables

-- Notification subscriptions
CREATE TABLE notification_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    subscription_type JSONB NOT NULL,
    delivery_methods JSONB NOT NULL,
    active BOOLEAN DEFAULT true,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    writer_id UUID REFERENCES writers(id)
);

-- Notification delivery history
CREATE TABLE notification_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID REFERENCES notification_subscriptions(id) ON DELETE CASCADE,
    event_id UUID REFERENCES events(id),
    notification_data JSONB NOT NULL,
    delivery_method TEXT NOT NULL,
    delivery_status TEXT NOT NULL, -- 'pending', 'sent', 'failed', 'retrying'
    attempts INT DEFAULT 0,
    last_error TEXT,
    delivered_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- User notification preferences
CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL UNIQUE,
    email TEXT,
    phone TEXT,
    telegram_chat_id TEXT,
    discord_user_id TEXT,
    slack_user_id TEXT,
    push_tokens JSONB, -- Array of {platform, token} objects
    timezone TEXT DEFAULT 'UTC',
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    max_notifications_per_hour INT DEFAULT 100,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Notification templates
CREATE TABLE notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    delivery_method TEXT NOT NULL,
    template_subject TEXT,
    template_body TEXT NOT NULL,
    template_data JSONB, -- Additional template configuration
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Rate limiting for notifications
CREATE TABLE notification_rate_limits (
    user_id TEXT NOT NULL,
    delivery_method TEXT NOT NULL,
    window_start TIMESTAMP NOT NULL,
    count INT DEFAULT 0,
    PRIMARY KEY (user_id, delivery_method, window_start)
);

-- Webhook endpoints
CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID REFERENCES notification_subscriptions(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT, -- For HMAC signing
    headers JSONB, -- Custom headers
    max_retries INT DEFAULT 3,
    timeout_seconds INT DEFAULT 30,
    active BOOLEAN DEFAULT true,
    last_success_at TIMESTAMP,
    last_failure_at TIMESTAMP,
    failure_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Notification filters for advanced subscriptions
CREATE TABLE notification_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID REFERENCES notification_subscriptions(id) ON DELETE CASCADE,
    filter_type TEXT NOT NULL, -- 'operator', 'amount', 'slasher', 'time', etc.
    filter_value JSONB NOT NULL,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Alert rules for automatic notifications
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    condition JSONB NOT NULL, -- Complex condition in JSON format
    severity TEXT NOT NULL, -- 'info', 'warning', 'critical'
    auto_subscribe_users BOOLEAN DEFAULT false,
    cooldown_minutes INT DEFAULT 60,
    last_triggered_at TIMESTAMP,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Notification groups for bulk notifications
CREATE TABLE notification_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    member_ids TEXT[], -- Array of user IDs
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_notification_subscriptions_user ON notification_subscriptions(user_id);
CREATE INDEX idx_notification_subscriptions_active ON notification_subscriptions(active) WHERE active = true;
CREATE INDEX idx_notification_deliveries_subscription ON notification_deliveries(subscription_id);
CREATE INDEX idx_notification_deliveries_status ON notification_deliveries(delivery_status);
CREATE INDEX idx_notification_deliveries_created ON notification_deliveries(created_at DESC);
CREATE INDEX idx_notification_preferences_user ON notification_preferences(user_id);
CREATE INDEX idx_webhook_endpoints_subscription ON webhook_endpoints(subscription_id);
CREATE INDEX idx_webhook_endpoints_active ON webhook_endpoints(active) WHERE active = true;
CREATE INDEX idx_notification_filters_subscription ON notification_filters(subscription_id);
CREATE INDEX idx_alert_rules_active ON alert_rules(active) WHERE active = true;
CREATE INDEX idx_notification_rate_limits_user_method ON notification_rate_limits(user_id, delivery_method);

-- Triggers
SELECT trigger_updated_at('notification_subscriptions');
SELECT trigger_updated_at('notification_deliveries');
SELECT trigger_updated_at('notification_preferences');
SELECT trigger_updated_at('notification_templates');
SELECT trigger_updated_at('webhook_endpoints');
SELECT trigger_updated_at('notification_filters');
SELECT trigger_updated_at('alert_rules');
SELECT trigger_updated_at('notification_groups');

-- Audit logging
SELECT trigger_audit_log('notification_subscriptions');
SELECT trigger_audit_log('notification_preferences');
SELECT trigger_audit_log('webhook_endpoints');

-- Functions for notification system

-- Function to check rate limits
CREATE OR REPLACE FUNCTION check_notification_rate_limit(
    p_user_id TEXT,
    p_delivery_method TEXT,
    p_max_per_hour INT DEFAULT 100
) RETURNS BOOLEAN AS $$
DECLARE
    v_count INT;
    v_window_start TIMESTAMP;
BEGIN
    v_window_start := date_trunc('hour', NOW());
    
    SELECT count INTO v_count
    FROM notification_rate_limits
    WHERE user_id = p_user_id
      AND delivery_method = p_delivery_method
      AND window_start = v_window_start;
    
    IF v_count IS NULL THEN
        INSERT INTO notification_rate_limits (user_id, delivery_method, window_start, count)
        VALUES (p_user_id, p_delivery_method, v_window_start, 1);
        RETURN TRUE;
    ELSIF v_count < p_max_per_hour THEN
        UPDATE notification_rate_limits
        SET count = count + 1
        WHERE user_id = p_user_id
          AND delivery_method = p_delivery_method
          AND window_start = v_window_start;
        RETURN TRUE;
    ELSE
        RETURN FALSE;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to get active subscriptions for an event
CREATE OR REPLACE FUNCTION get_matching_subscriptions(
    p_event_type TEXT,
    p_operator_id UUID DEFAULT NULL,
    p_amount BIGINT DEFAULT NULL
) RETURNS TABLE (
    subscription_id UUID,
    user_id TEXT,
    delivery_methods JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ns.id,
        ns.user_id,
        ns.delivery_methods
    FROM notification_subscriptions ns
    WHERE ns.active = true
      AND (
        -- Match all slashings
        ns.subscription_type->>'type' = 'all_slashings'
        -- Match specific operator
        OR (ns.subscription_type->>'type' = 'specific_operator' 
            AND ns.subscription_type->'registration_root' = to_jsonb(p_operator_id))
        -- Match high value slashings
        OR (ns.subscription_type->>'type' = 'high_value_slashings'
            AND p_amount >= (ns.subscription_type->>'min_amount')::BIGINT)
      );
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup old notification data
CREATE OR REPLACE FUNCTION cleanup_old_notifications(
    p_days_to_keep INT DEFAULT 30
) RETURNS INT AS $$
DECLARE
    v_deleted_count INT;
BEGIN
    DELETE FROM notification_deliveries
    WHERE created_at < NOW() - INTERVAL '1 day' * p_days_to_keep
      AND delivery_status IN ('sent', 'failed');
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    -- Also cleanup old rate limit records
    DELETE FROM notification_rate_limits
    WHERE window_start < NOW() - INTERVAL '1 day';
    
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;