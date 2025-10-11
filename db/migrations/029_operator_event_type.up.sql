-- =============================================================================
-- OPERATOR EVENT TYPE
-- =============================================================================

CREATE TYPE operator_event_type AS ENUM (
    'OperatorRegistered',
    'OperatorSlashed',
    'OperatorUnregistered',
    'CollateralClaimed',
    'CollateralAdded',
    'OperatorOptedIn',
    'OperatorOptedOut'
);
