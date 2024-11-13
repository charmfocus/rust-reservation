# Core Reservation

- Feature Name: core-reservation
- Start Date: 2024-11-12 12:44:30

## Summary

A core reservation service that solves the problem of reserving a resource for a period of time. We leverage postgres EXCLUDE constraints to ensure that only one reservation can be made for a given resource at a given time.

## Motivation

We need a common solution for various reservation requirements: 1) calendar booking; 2) hotel/room booking; 3) meeting room booking; 4) parking lot booking; 5) etc. Repeatedly building features for these requirements is a waste of time and resources. We should have a common solution that can be used by all teams.

## Guide-level explanation

### Service interface

We would use gRPC as a service interface. Below is the proto definition.

```proto
enum ReservationStatus {
  PENDING = 0;
  CANCELED = 1;
  BLOCKED = 2;
}

enum ReservationUpdateType {
  UNKNOWN = 0;
  INSERT = 1;
  UPDATE = 2;
  DELETE = 3;
}

message Reservation {
  string id = 1;
  string user_id = 2;
  ReservationStatus type = 3;

  // resource reservation window
  string resource_id = 4;
  google.protobuf.Timestamp start_time = 5;
  google.protobuf.Timestamp end_time = 6;

  // extra note
  string note = 7;
}

message ReserveRequest {
  Reservation reservation = 1;
}

message ReserveResponse {
  Reservation reservation = 1;
}

message ConfirmRequest {
  Reservation reservation = 1;
}

message ConfirmResponse {
  Reservation reservation = 1;
}

message UpdateRequest {
  string note = 2;
}

message UpdateResponse {
  Reservation reservation = 1;
}

message CancelRequest {
  string id = 1;
}

message CancelResponse {
  Reservation reservation = 1;
}

message GetRequest {
  string id = 1;
}

message GetResponse {
  Reservation reservation = 1;
}

message QueryRequest {
  string resource_id = 1;
  string user_id = 2;
  // use status to filter results. If UNKNOWN, all results are returned.
  ReservationStatus status = 3;
  google.protobuf.Timestamp start_time = 4;
  google.protobuf.Timestamp end_time = 5;
}

message ListenRequest {
  // empty
}

message ListenResponse {
  int64 id = 1;
  Reservation reservation = 1;
}

service ReservationService {
  rpc Reserve(ReserveRequest) returns (ReserveResponse);
  rpc Confirm(ConfirmRequest) returns (ConfirmResponse);
  rpc Update(UpdateRequest) returns (UpdateResponse);
  rpc Cancel(CancelRequest) returns (CancelResponse);
  rpc Get(GetRequest) returns (GetResponse);
  rpc Query(QueryRequest) returns (stream Reservation);
  // another system could monitor newly added/confirmed/canceled reservations
  rpc Listen(ListenRequest) returns (stream Reservation);
}
```

### Database schema

We use postgres as the database. Below is the schema for the reservation table.

```sql
CREATE SCHEMA rsvp;
CREATE TYPE rsvp.reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked', 'canceled');
CREATE TYPE rsvp.reservation_update_type AS ENUM ('unknown', 'insert', 'update', 'delete');
CREATE TABLE rsvp.reservations (
  id uuid NOT NULL DEFAULT uuid_generate_v4(),
  user_id VARCHAR(64) NOT NULL,
  status rsvp.reservation_status NOT NULL DEFAULT 'pending',
  resource_id VARCHAR(64) NOT NULL,
  timespan TSTZRANGE NOT NULL,
  note TEXT,
  CONSTRAINT reservations_pkey PRIMARY KEY (id),
  CONSTRAINT reservations_conflict EXCLUDE USING GIST (resource_id WITH =, timespan WITH &&)
);
CREATE INDEX reservations_resource_id_idx ON rsvp.reservations (resource_id);
CREATE INDEX reservations_user_id_idx ON rsvp.reservations (user_id);

-- if user_id is null, find all reservations within during for the resource
-- if resource_id is null, find all reservations within during for the user
-- if both are null, find all reservations within during
-- if both set, find all reservations within during for the resource and user
CREATE OR REPLACE FUNCTION rsvp.query(user_id text, rid text, during: TSTZRANGE, ts: TSTZRANGE) RETURNS TABLE rsvp.reservations AS $$ $$ LANGUAGE plpgsql;

-- reservation change queue
CREATE TABLE rsvp.reservation_changes (
    id SERIAL NOT NULL,
    reservation_id UUID NOT NULL,
    op rsvp.reservation_update_type NOT NULL,
);

-- trigger for add/update/delete a reservation
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    -- update reservation_changes
    INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'insert');
  ELSIF TG_OP = 'UPDATE' THEN
    -- if status changed, update reservation_changes
    IF OLD.status <> NEW.status THEN
      INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'update');
    END IF;
  ELSIF TG_OP = 'DELETE' THEN
    -- update reservation_changes
    INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (OLD.id, 'delete');
  END IF;
  -- notify a channel called reservation_update
  NOTIFY reservation_update;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER reservations_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();
```

Here we use EXCLUDE constraint provided by postgres to ensure that on overlapping reservations can not be made for a given resource at a given time.

```sql
CONSTRAINT reservations_conflict EXCLUDE USING GIST (resource_id WITH =, timespan WITH &&)
```

We also use a trigger to notify a channel called `reservation_update` whenever a reservation is added/updated/deleted. To make sure even we missed certain messages from the channel when DB connection is down for some reason, we use a queue to store reservation changes. Thus when we receive a notification, we can query the queue to get all the changes since last time we checked, and once we finished processing all the changes, we can delete them from the queue.

## Reference-level explanation

TDB

## Drawbacks

N/A

## Rationale and alternatives

N/A

## Prior art

N/A

## Unresolved questions

- how to handle repeated reservations? - is this more ore less a business logic which shouldn't be put into this layer?(non-goal: we consider this a business logic and should be handled by the caller)
- if load is big, we may use an external queue for recording changes.
- we haven't considered observability/deployment yet.
- query performance might be an issue - need to revisit the index and also consider using cache.

## Future possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.

```

```

```

```
