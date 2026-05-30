-- Playing XI per team per match
CREATE TABLE IF NOT EXISTS match_playing_xi (
    id BIGSERIAL PRIMARY KEY,
    match_id BIGINT NOT NULL,
    team_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (match_id, team_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_match_playing_xi_match ON match_playing_xi (match_id);
CREATE INDEX IF NOT EXISTS idx_match_playing_xi_team ON match_playing_xi (match_id, team_id);

-- Per-match player stats (aggregated from ball-by-ball progress)
CREATE TABLE IF NOT EXISTS match_player_stats (
    id BIGSERIAL PRIMARY KEY,
    match_id BIGINT NOT NULL,
    player_id BIGINT NOT NULL,
    runs_scored INT NOT NULL DEFAULT 0,
    balls_faced INT NOT NULL DEFAULT 0,
    fours INT NOT NULL DEFAULT 0,
    sixes INT NOT NULL DEFAULT 0,
    is_out BOOLEAN NOT NULL DEFAULT FALSE,
    wickets_taken INT NOT NULL DEFAULT 0,
    balls_bowled INT NOT NULL DEFAULT 0,
    runs_conceded INT NOT NULL DEFAULT 0,
    UNIQUE (match_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_match_player_stats_match ON match_player_stats (match_id);
