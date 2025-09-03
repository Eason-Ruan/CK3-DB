use sqlx::{SqlitePool, Error as SqlxError};

pub struct DatabaseSchema {
    pool: SqlitePool,
}

impl DatabaseSchema {
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// 创建所有数据库表
    pub async fn create_all_tables(&self) -> Result<(), SqlxError> {
        self.create_characters_table().await?;
        self.create_titles_table().await?;
        self.create_faiths_table().await?;
        self.create_cultures_table().await?;
        self.create_dynasties_table().await?;
        self.create_houses_table().await?;
        self.create_artifacts_table().await?;
        self.create_memories_table().await?;
        
        // 关系表
        self.create_character_traits_table().await?;
        self.create_character_titles_table().await?;
        self.create_character_relationships_table().await?;
        self.create_character_kills_table().await?;
        self.create_title_claims_table().await?;
        self.create_title_history_table().await?;
        self.create_faith_tenets_table().await?;
        self.create_faith_doctrines_table().await?;
        self.create_culture_traditions_table().await?;
        self.create_dynasty_perks_table().await?;
        
        // 系统表
        self.create_game_metadata_table().await?;
        self.create_localization_keys_table().await?;
        
        Ok(())
    }

    async fn create_characters_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                nick TEXT,
                birth_date TEXT NOT NULL,
                death_date TEXT,
                death_reason TEXT,
                is_female BOOLEAN NOT NULL,
                is_dead BOOLEAN NOT NULL DEFAULT 0,
                
                gold REAL DEFAULT 0.0,
                piety REAL DEFAULT 0.0,
                prestige REAL DEFAULT 0.0,
                dread REAL DEFAULT 0.0,
                strength REAL DEFAULT 0.0,
                
                diplomacy_skill INTEGER DEFAULT 0,
                martial_skill INTEGER DEFAULT 0,
                stewardship_skill INTEGER DEFAULT 0,
                learning_skill INTEGER DEFAULT 0,
                intrigue_skill INTEGER DEFAULT 0,
                prowess_skill INTEGER DEFAULT 0,
                
                faith_id INTEGER,
                culture_id INTEGER,
                house_id INTEGER,
                liege_id INTEGER,
                
                dna TEXT,
                languages TEXT,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_faith ON characters(faith_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_culture ON characters(culture_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_house ON characters(house_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_liege ON characters(liege_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_birth_date ON characters(birth_date)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_characters_death_date ON characters(death_date)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_titles_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS titles (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                tier TEXT NOT NULL,
                
                color_red INTEGER DEFAULT 70,
                color_green INTEGER DEFAULT 255,
                color_blue INTEGER DEFAULT 70,
                
                de_jure_liege_id INTEGER,
                de_facto_liege_id INTEGER,
                capital_id INTEGER,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_titles_key ON titles(key)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_titles_tier ON titles(tier)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_titles_de_jure_liege ON titles(de_jure_liege_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_titles_de_facto_liege ON titles(de_facto_liege_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_faiths_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS faiths (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                fervor REAL DEFAULT 0.0,
                
                head_title_id INTEGER,
                head_id INTEGER,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_faiths_head ON faiths(head_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_cultures_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cultures (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                ethos TEXT,
                heritage TEXT NOT NULL,
                martial_custom TEXT NOT NULL,
                language TEXT NOT NULL,
                created_date TEXT,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        Ok(())
    }

    async fn create_dynasties_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dynasties (
                id INTEGER PRIMARY KEY,
                name TEXT,
                prestige_total REAL DEFAULT 0.0,
                prestige_current REAL DEFAULT 0.0,
                
                leader_id INTEGER,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_dynasties_leader ON dynasties(leader_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_houses_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS houses (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                motto TEXT,
                found_date TEXT,
                
                dynasty_id INTEGER NOT NULL,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_houses_dynasty ON houses(dynasty_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_artifacts_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS artifacts (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                rarity TEXT NOT NULL,
                type TEXT NOT NULL,
                
                owner_id INTEGER,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_artifacts_owner ON artifacts(owner_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_memories_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY,
                type TEXT NOT NULL,
                date TEXT NOT NULL,
                
                character_id INTEGER NOT NULL,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_character ON memories(character_id)").execute(&self.pool).await?;

        Ok(())
    }

    // 关系表
    async fn create_character_traits_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS character_traits (
                character_id INTEGER NOT NULL,
                trait_name TEXT NOT NULL,
                PRIMARY KEY (character_id, trait_name)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_traits_character ON character_traits(character_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_traits_trait ON character_traits(trait_name)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_character_titles_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS character_titles (
                character_id INTEGER NOT NULL,
                title_id INTEGER NOT NULL,
                is_primary BOOLEAN DEFAULT 0,
                PRIMARY KEY (character_id, title_id)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_titles_character ON character_titles(character_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_titles_title ON character_titles(title_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_character_relationships_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS character_relationships (
                character_id INTEGER NOT NULL,
                related_character_id INTEGER NOT NULL,
                relationship_type TEXT NOT NULL,
                PRIMARY KEY (character_id, related_character_id, relationship_type)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_relationships_character ON character_relationships(character_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_relationships_related ON character_relationships(related_character_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_character_kills_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS character_kills (
                killer_id INTEGER NOT NULL,
                victim_id INTEGER NOT NULL,
                PRIMARY KEY (killer_id, victim_id)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_kills_killer ON character_kills(killer_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_character_kills_victim ON character_kills(victim_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_title_claims_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS title_claims (
                title_id INTEGER NOT NULL,
                claimant_id INTEGER NOT NULL,
                claim_type TEXT DEFAULT 'normal',
                PRIMARY KEY (title_id, claimant_id)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_title_claims_title ON title_claims(title_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_title_claims_claimant ON title_claims(claimant_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_title_history_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS title_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                holder_id INTEGER,
                action TEXT NOT NULL,
                
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_title_history_title ON title_history(title_id)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_title_history_date ON title_history(date)").execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_title_history_holder ON title_history(holder_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_faith_tenets_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS faith_tenets (
                faith_id INTEGER NOT NULL,
                tenet_name TEXT NOT NULL,
                PRIMARY KEY (faith_id, tenet_name)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_faith_tenets_faith ON faith_tenets(faith_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_faith_doctrines_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS faith_doctrines (
                faith_id INTEGER NOT NULL,
                doctrine_name TEXT NOT NULL,
                PRIMARY KEY (faith_id, doctrine_name)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_faith_doctrines_faith ON faith_doctrines(faith_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_culture_traditions_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS culture_traditions (
                culture_id INTEGER NOT NULL,
                tradition_name TEXT NOT NULL,
                PRIMARY KEY (culture_id, tradition_name)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_culture_traditions_culture ON culture_traditions(culture_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_dynasty_perks_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dynasty_perks (
                dynasty_id INTEGER NOT NULL,
                perk_name TEXT NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (dynasty_id, perk_name)
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_dynasty_perks_dynasty ON dynasty_perks(dynasty_id)").execute(&self.pool).await?;

        Ok(())
    }

    async fn create_game_metadata_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS game_metadata (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                current_date TEXT NOT NULL,
                start_date TEXT NOT NULL,
                save_file_name TEXT,
                game_version TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        Ok(())
    }

    async fn create_localization_keys_table(&self) -> Result<(), SqlxError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS localization_keys (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                language TEXT DEFAULT 'english',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).execute(&self.pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_localization_keys_language ON localization_keys(language)").execute(&self.pool).await?;

        Ok(())
    }
}
