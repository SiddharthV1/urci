-- DB supports multiple writers
-- writers table will store the db writers information
 create table "writers" (
   "id" uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
   "name" text not null,
   "session" text not null,
   "created_at" timestamp default (now()),
   "updated_at" timestamp default (now())
 );

-- Audit Log table that will store the changes made to the tables
create table audit_log (
    id uuid PRIMARY KEY DEFAULT (gen_random_uuid()),
    action varchar(255) not null,
    table_name varchar(255) not null,
    record_id int not null,
    old_data jsonb,
    new_data jsonb,
    created_at timestamp default now(),
    writer_id uuid
);

ALTER TABLE "audit_log" ADD FOREIGN KEY ("writer_id") REFERENCES "writers" ("id");

-- Audit Log trigger function	
create or replace function audit_log_trigger()
    returns trigger as
$$
declare
    action varchar(255);
begin
    if TG_OP = 'INSERT' then
	action := 'INSERT';
    elsif TG_OP = 'UPDATE' then
	action := 'UPDATE';
    elsif TG_OP = 'DELETE' then
	action := 'DELETE';
    end if;

    insert into audit_log (action, table_name, record_id, old_data, new_data, created_at)
    values (action, TG_TABLE_NAME, coalesce(OLD.id, NEW.id), row_to_json(OLD), row_to_json(NEW), now());

    return NEW;
end;
$$ language plpgsql;

-- Function to attach the audit log trigger to a table 
CREATE OR REPLACE FUNCTION trigger_audit_log(tablename regclass)
RETURNS void AS
$$
BEGIN
    EXECUTE format('
        CREATE TRIGGER audit_log_trigger
        AFTER INSERT OR UPDATE OR DELETE ON %s
        FOR EACH ROW
        EXECUTE FUNCTION audit_log_trigger();', tablename);
END;
$$ LANGUAGE plpgsql;


--  trigger to set updated_at column
create or replace function set_updated_at()
    returns trigger as
$$
begin
    NEW.updated_at = now();
    return NEW;
end;
$$ language plpgsql;

-- Function to attach the set_updated_at trigger to a table
create or replace function trigger_updated_at(tablename regclass)
    returns void as
$$
begin
    execute format('CREATE TRIGGER set_updated_at
        BEFORE UPDATE
        ON %s
        FOR EACH ROW
        WHEN (OLD is distinct from NEW)
    EXECUTE FUNCTION set_updated_at();', tablename);
end;
$$ language plpgsql;


