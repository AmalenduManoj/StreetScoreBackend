UPDATE players
SET matches_played = 0
WHERE matches_played IS NULL;

ALTER TABLE players
    ALTER COLUMN matches_played SET DEFAULT 0,
    ALTER COLUMN matches_played SET NOT NULL;