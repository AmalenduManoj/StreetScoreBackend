 # Cricscore Backend

Cricscore Backend is an Actix-web + SQLx Rust service providing APIs for cricket tournament management and live ball-by-ball scoring.

## Features
- User authentication (JWT)
- Team and Player management
- Tournament scheduling and team registration
- Ball-by-ball `progress` tracking (deliveries)
- Tournament standings and leaderboards
- Batsman and Bowler rankings
- Tournament match registry and full match details

## Prerequisites
- Rust (stable) and Cargo
- Postgres (12+)
- `DATABASE_URL` environment variable pointing to your Postgres DB

Example .env:

DATABASE_URL=postgres://user:password@localhost:5432/cricscoredb

## Run locally

1. Set `DATABASE_URL` env var.
2. Run migrations in `migrations/` or create schema manually.
3. Build & run:

```bash
cargo build
cargo run
```

The server listens on `127.0.0.1:8080` by default.

## Authorization
Most protected endpoints require a JWT bearer token in the `Authorization` header:

```
Authorization: Bearer <TOKEN>
```

Public endpoints: signup, login, fetching tournaments, teams, matches, players.

Generate token via `/auth/login` (returns token in response).

---

## Database Models (important tables)

Progress (ball-by-ball tracking):

```sql
CREATE TABLE progress (
    id BIGSERIAL PRIMARY KEY,
    match_id BIGINT NOT NULL,
    batter_id BIGINT NOT NULL,
    bowler_id BIGINT NOT NULL,
    runs_scored INTEGER NOT NULL DEFAULT 0,
    is_wicket BOOLEAN DEFAULT FALSE,
    over_number INTEGER NOT NULL,
    ball_number INTEGER NOT NULL,
    commentary VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (match_id) REFERENCES matches(id),
    FOREIGN KEY (batter_id) REFERENCES players(id),
    FOREIGN KEY (bowler_id) REFERENCES players(id)
);
```

Tournament standing:

```sql
CREATE TABLE tournament_standing (
    id BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL,
    team_id BIGINT NOT NULL,
    match_played INTEGER NOT NULL DEFAULT 0,
    wons INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    points INTEGER NOT NULL DEFAULT 0,
    run_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    UNIQUE(tournament_id, team_id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (team_id) REFERENCES teams(id)
);
```

Batsman ranking:

```sql
CREATE TABLE batsman_ranking (
    id BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    runs INTEGER NOT NULL DEFAULT 0,
    ball_faced INTEGER NOT NULL DEFAULT 0,
    no_of_outs INTEGER NOT NULL DEFAULT 0,
    UNIQUE(tournament_id, player_id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (player_id) REFERENCES players(id)
);
```

Bowler ranking:

```sql
CREATE TABLE bowler_ranking (
    id BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    runs_given INTEGER NOT NULL DEFAULT 0,
    ball_bowled INTEGER NOT NULL DEFAULT 0,
    wickets INTEGER NOT NULL DEFAULT 0,
    UNIQUE(tournament_id, player_id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (player_id) REFERENCES players(id)
);
```

Tournament match registry:

```sql
CREATE TABLE tournament_match (
    id BIGSERIAL PRIMARY KEY,
    tournament_id BIGINT NOT NULL,
    match_id BIGINT NOT NULL,
    match_number INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tournament_id, match_id),
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id),
    FOREIGN KEY (match_id) REFERENCES matches(id)
);
```

Other important existing tables (brief): `users`, `teams`, `players`, `matches`, `team_player_registry`, `team_tournament_registry`.

---

## API Endpoints

All endpoints below are relative to `http://127.0.0.1:8080`.

### Auth

- POST `/auth/signup` — Create user
  - Payload: `{ "email": "a@b.com", "password": "pass" }`
  - Response: `token` and `user` object

- POST `/auth/login` — Login
  - Payload: `{ "email": "a@b.com", "password": "pass" }`
  - Response: `{ token, user }`

### Matches (public)

- GET `/matches` — list matches
- GET `/matches/live` — live matches
- GET `/matches/{id}` — match by id

### Players & Teams (public & protected)

- GET `/players` — list players
- GET `/players/stats/{id}` — player stats
- POST `/players` — create player (protected)
- PUT `/players/{id}` — update player (protected)
- GET `/teams` — list teams
- GET `/teams/{id}` — get team
- POST `/team_players` endpoints exist to add/remove players (protected)

### Progress (Ball-by-ball, public GETs)

- POST `/api/progress` — create delivery
  - Payload example:
```json
{
  "match_id": 1,
  "batter_id": 5,
  "bowler_id": 8,
  "runs_scored": 4,
  "is_wicket": false,
  "over_number": 5,
  "ball_number": 3,
  "commentary": "Boundary",
  "created_at": "2026-05-18T10:30:00Z"
}
```

- GET `/api/progress/match/{match_id}` — all deliveries for match
- GET `/api/progress/match/{match_id}/over/{over_number}` — deliveries in over
- GET `/api/progress/{id}` — delivery by id
- PUT `/api/progress/{id}` — update delivery (protected)
- DELETE `/api/progress/{id}` — delete delivery (protected)
- GET `/api/progress/match/{match_id}/summary` — match aggregate stats

### Tournament Standings (public GETs)

- GET `/api/tournament/{tournament_id}/standings` — points table
- GET `/api/tournament/{tournament_id}/standings/{team_id}` — team standing
- POST `/api/tournament/{tournament_id}/standings` — create standing (protected)
- PUT `/api/tournament/{tournament_id}/standings/{team_id}` — update standing (protected)
- GET `/api/tournament/{tournament_id}/leaderboard` — top teams

### Rankings (public GETs)

- GET `/api/tournament/{tournament_id}/rankings/batsmen` — batsman rankings
- GET `/api/tournament/{tournament_id}/rankings/batsmen/{player_id}` — batsman detail
- POST `/api/tournament/{tournament_id}/rankings/batsmen` — create (protected)
- PUT `/api/tournament/{tournament_id}/rankings/batsmen/{player_id}` — update (protected)

- GET `/api/tournament/{tournament_id}/rankings/bowlers` — bowler rankings
- GET `/api/tournament/{tournament_id}/rankings/bowlers/{player_id}` — bowler detail
- POST `/api/tournament/{tournament_id}/rankings/bowlers` — create (protected)
- PUT `/api/tournament/{tournament_id}/rankings/bowlers/{player_id}` — update (protected)

### Tournament Match Registry

- POST `/api/tournament/{tournament_id}/matches` — create tournament match (protected)
  - Payload:
```json
{
  "match_number": 1,
  "match_data": {
    "team1_id": 1,
    "team2_id": 2,
    "venue": "Main Ground",
    "total_overs": 20,
    "team1_score": 0,
    "team1_wickets": 0,
    "team1_overs": 0,
    "team2_score": 0,
    "team2_wickets": 0,
    "team2_overs": 0,
    "status": "scheduled"
  }
}
```
- GET `/api/tournament/{tournament_id}/matches` — list tournament matches with embedded match details
- GET `/api/tournament/{tournament_id}/matches/{match_number}` — get one tournament match with embedded match details
- GET `/api/tournament/match/{id}` — get a tournament match by registry id with embedded match details
- PUT `/api/tournament/match/{id}` — update (protected)
- DELETE `/api/tournament/match/{id}` — delete (protected)
- GET `/api/tournament/{tournament_id}/matches/{match_number}/details` — full match details

### Docs

- GET `/api/docs` — returns project README/Docs (this file) as markdown

---

## Notes & Conventions
- All timestamps use UTC ISO-8601
- Points system: default 2 for win, 1 for tie/no-result, 0 for loss
- Calculate strike-rate and economy as shown in handlers
