-- Create the JackpotPlayers table first, assuming 'users' table exists
CREATE TABLE JackpotPlayers (
    player_id INT PRIMARY KEY,  -- Assuming player_id is unique
    amount FLOAT,
    session_id INT,
    FOREIGN KEY (player_id) REFERENCES users(id)  -- Ensure 'users' table exists
);

-- Create the JackpotGames table
CREATE TABLE JackpotGames (
    game_id SERIAL PRIMARY KEY,  -- Use SERIAL for auto-increment (PostgreSQL)
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    status VARCHAR(20) CHECK (status IN ('ongoing', 'completed')) DEFAULT 'ongoing',
    winner_id INT,
    FOREIGN KEY (winner_id) REFERENCES JackpotPlayers(player_id)  -- Ensure JackpotPlayers table exists
);

-- Create the foreign key constraint in JackpotPlayers table
ALTER TABLE JackpotPlayers
ADD CONSTRAINT fk_session
FOREIGN KEY ( ) REFERENCES JackpotGames(game_id);
