-- title table init
CREATE TABLE IF NOT EXISTS titles (
    id INTEGER,                    -- 头衔ID (GameId)
    key TEXT NOT NULL UNIQUE,                 -- 头衔键值（如 "e_byzantium"）
    name TEXT NOT NULL,                       -- 头衔名称（已本地化）
    tier TEXT NOT NULL,                       -- 头衔等级（empire, kingdom, duchy, county, barony）
    
    -- 颜色信息
    color_red INTEGER DEFAULT 70,
    color_green INTEGER DEFAULT 255,
    color_blue INTEGER DEFAULT 70,
    
    -- 外键引用
    de_jure_liege_id INTEGER,  -- 法理领主
    de_facto_liege_id INTEGER, -- 实际领主
    county_culture INTEGER,
    county_faith INTEGER,
    capital_id INTEGER,        -- 首都

    meta_id INTEGER REFERENCES game_metadata(id),

    PRIMARY KEY (meta_id, id),

    FOREIGN KEY (meta_id, de_jure_liege_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, de_facto_liege_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, capital_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, county_culture) REFERENCES cultures(meta_id, id),
    FOREIGN KEY (meta_id, county_faith) REFERENCES faiths(meta_id, id)
);

-- index for titles
CREATE INDEX IF NOT EXISTS idx_titles_key ON titles(meta_id, key);
CREATE INDEX IF NOT EXISTS idx_titles_tier ON titles(meta_id, tier);
CREATE INDEX IF NOT EXISTS idx_titles_de_jure_liege ON titles(meta_id, de_jure_liege_id);
CREATE INDEX IF NOT EXISTS idx_titles_de_facto_liege ON titles(meta_id, de_facto_liege_id);
CREATE INDEX IF NOT EXISTS idx_titles_meta_id ON titles(meta_id, id);

CREATE TABLE IF NOT EXISTS titles_facto_vassals (
    title_id        INTEGER,
    vassal_title_id INTEGER,
    meta_id         INTEGER REFERENCES game_metadata (id),
    PRIMARY KEY (meta_id, title_id, vassal_title_id),

    FOREIGN KEY (meta_id, title_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, vassal_title_id) REFERENCES titles(meta_id, id)
);

CREATE TABLE IF NOT EXISTS titles_jure_vassals (
    title_id        INTEGER NOT NULL,
    vassal_title_id INTEGER NOT NULL,
    meta_id         INTEGER REFERENCES game_metadata (id),
    PRIMARY KEY (meta_id, title_id, vassal_title_id),
    FOREIGN KEY (meta_id, title_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, vassal_title_id) REFERENCES titles(meta_id, id)
);


-- faith table init
CREATE TABLE IF NOT EXISTS faiths (
    id INTEGER,        -- 信仰ID (GameId)
    name TEXT NOT NULL,                       -- 信仰名称（已本地化）
    fervor REAL DEFAULT 0.0,                  -- 狂热度
    
    -- 外键引用
    head_title_id INTEGER, -- 宗教领袖头衔
    head_id INTEGER,   -- 宗教领袖

    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, id),

    Foreign Key (meta_id, head_title_id) REFERENCES titles(meta_id, id),
    Foreign Key (meta_id, head_id) REFERENCES characters(meta_id, id)
);

-- index for faiths
CREATE INDEX IF NOT EXISTS idx_faiths_head ON faiths(meta_id, head_id);

-- culture table init
CREATE TABLE IF NOT EXISTS cultures (
    id INTEGER,                    -- 文化ID (GameId)
    name TEXT NOT NULL,                       -- 文化名称（已本地化）
    ethos TEXT,                               -- 文化理念（已本地化）
    heritage TEXT NOT NULL,                   -- 文化传承（已本地化）
    martial TEXT NOT NULL,             -- 军事传统（已本地化）
    language TEXT NOT NULL,                   -- 语言（已本地化）
    date TEXT,

    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, id)
);

-- dynasty table init
CREATE TABLE IF NOT EXISTS dynasties (
    id INTEGER,                    -- 王朝ID (GameId)
    name TEXT,                                -- 王朝名称（已本地化）
    prestige_total REAL DEFAULT 0.0,          -- 总威望
    prestige_current REAL DEFAULT 0.0,        -- 当前威望
    
    -- 外键引用
    leader_id INTEGER, -- 王朝领袖
    
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, id),

    FOREIGN KEY (meta_id, leader_id) REFERENCES characters(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_dynasties_leader ON dynasties(meta_id, leader_id);

-- dynasty house relationship table init
CREATE TABLE IF NOT EXISTS dynasty_houses
(
    dynasty_id INTEGER NOT NULL,
    house_id   BIGINT  NOT NULL,

    meta_id    INTEGER REFERENCES game_metadata (id),

    PRIMARY KEY (meta_id, dynasty_id, house_id),

    FOREIGN KEY (meta_id, dynasty_id) REFERENCES dynasties(meta_id, id),
    FOREIGN KEY (meta_id, house_id) REFERENCES houses(meta_id, id)
);

-- house table init
CREATE TABLE IF NOT EXISTS houses (
    id INTEGER,                    -- 家族ID (GameId)
    name TEXT NOT NULL,                       -- 家族名称（已本地化）
    found_date DATE,                          -- 建立日期
    
    -- 外键引用

    meta_id   INTEGER REFERENCES game_metadata (id),

    PRIMARY KEY (meta_id, id)
);

-- house leaders table init
CREATE TABLE IF NOT EXISTS houses_leaders (
    house_id INTEGER,
    leader_id INTEGER,
    meta_id    INTEGER REFERENCES game_metadata (id),

    PRIMARY KEY (meta_id, house_id, leader_id),

    FOREIGN KEY (meta_id, house_id) REFERENCES houses(meta_id, id),
    FOREIGN KEY (meta_id, leader_id) REFERENCES characters(meta_id, id)
);

-- house motto table init
CREATE TABLE IF NOT EXISTS houses_mottos (
    motto TEXT,
    time INTEGER,
    by_name TEXT,
    meta_id INTEGER REFERENCES  game_metadata (id),
    PRIMARY KEY (meta_id, motto, time, by_name)
);
-- artifact table init
CREATE TABLE IF NOT EXISTS artifacts (
    id INTEGER,                    -- 神器ID (GameId)
    name TEXT NOT NULL,                       -- 神器名称（已本地化）
    description TEXT NOT NULL,                -- 神器描述（已本地化）
    rarity TEXT NOT NULL,                     -- 稀有度（已本地化）
    type TEXT NOT NULL,                       -- 类型（已本地化）

    quality INTEGER NOT NULL,
    wealth INTEGER NOT NULL,
    owner_id INTEGER, -- 拥有者

    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID
    PRIMARY KEY (meta_id, id),

    FOREIGN KEY (meta_id, owner_id) REFERENCES characters(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_artifacts_owner ON artifacts(meta_id, owner_id);

-- memory table init
CREATE TABLE IF NOT EXISTS memories (
    id INTEGER,                    -- 记忆ID (GameId)
    type TEXT NOT NULL,                       -- 记忆类型（-- 已本地化）
    date DATE NOT NULL,                       -- 记忆日期
    
    -- 外键引用
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, id)
);

CREATE TABLE IF NOT EXISTS memory_participants (
    memory_id INTEGER NOT NULL,
    character_id BIGINT NOT NULL,
    role TEXT NOT NULL,                       -- 角色在记忆中的角色（已本地化）
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, memory_id, character_id),

    FOREIGN KEY (meta_id, memory_id) REFERENCES memories(meta_id, id),
    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id)
);

CREATE INDEX IF NOT EXISTS idx_memory_participants_character ON memory_participants(meta_id, character_id);

CREATE TABLE IF NOT EXISTS memory_variables (
    memory_id INTEGER NOT NULL,
    var_name TEXT NOT NULL,                   -- 变量名称
    var_type TEXT NOT NULL,                   -- 变量类型
    var_value TEXT NOT NULL,                  -- 变量值
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, memory_id, var_name),
    FOREIGN KEY (meta_id, memory_id) REFERENCES memories(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_memories_character ON memories(meta_id);

-- character table init
CREATE TABLE IF NOT EXISTS characters (
    id INTEGER,                    -- 角色ID (GameId)
    name TEXT NOT NULL,                       -- 角色名称（已本地化）
    nick TEXT,                                -- 绰号（已本地化）
    birth_date DATE NOT NULL,                 -- 出生日期
    death_date DATE,                          -- 死亡日期
    death_reason TEXT,                        -- 死亡原因（已本地化）
    is_female BOOLEAN NOT NULL,               -- 性别
    is_dead BOOLEAN NOT NULL DEFAULT FALSE,   -- 是否死亡
    
    -- 基础属性
    gold REAL DEFAULT 0.0,                    -- 金钱
    piety REAL DEFAULT 0.0,                   -- 虔诚度
    prestige REAL DEFAULT 0.0,                -- 威望
    dread REAL DEFAULT 0.0,                   -- 恐怖值
    strength REAL DEFAULT 0.0,                -- 个人武力
    
    -- 技能值（外交、军事、管理、学识、密谋、勇武）
    diplomacy_skill INTEGER DEFAULT 0,
    martial_skill INTEGER DEFAULT 0,
    stewardship_skill INTEGER DEFAULT 0,
    learning_skill INTEGER DEFAULT 0,
    intrigue_skill INTEGER DEFAULT 0,
    prowess_skill INTEGER DEFAULT 0,
    
    -- 外键引用
    faith_id INTEGER,    -- 信仰
    culture_id INTEGER, -- 文化
    house_id INTEGER,    -- 家族
    liege_id INTEGER, -- 领主

    -- 其他数据
    dna TEXT,                                 -- DNA字符串
    
    meta_id INTEGER REFERENCES game_metadata(id),

    PRIMARY KEY (meta_id, id),

    FOREIGN KEY (meta_id, faith_id) REFERENCES faiths(meta_id, id),
    FOREIGN KEY (meta_id, culture_id) REFERENCES cultures(meta_id, id),
    FOREIGN KEY (meta_id, house_id) REFERENCES houses(meta_id, id),
    FOREIGN KEY (meta_id, liege_id) REFERENCES characters(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_characters_birth_date ON characters(meta_id, birth_date);
CREATE INDEX IF NOT EXISTS idx_characters_death_date ON characters(meta_id, death_date);

-- character trait table init
CREATE TABLE IF NOT EXISTS character_traits (
    character_id INTEGER NOT NULL,
    trait_name TEXT NOT NULL,                 -- 特质名称（已本地化）
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, character_id, trait_name),
    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id)
);

-- character title table init
CREATE TABLE IF NOT EXISTS character_titles (
    character_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE,         -- 是否为主要头衔
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (character_id, title_id),
    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, title_id) REFERENCES titles(meta_id, id)
);

-- character relationship table init
CREATE TABLE IF NOT EXISTS character_relationships (
    character_id INTEGER NOT NULL,
    related_character_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL,          -- 关系类型：spouse, child, parent, former_spouse
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, character_id, related_character_id, relationship_type),
    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, related_character_id) REFERENCES characters(meta_id, id)
);

-- character vassals relational table init
CREATE TABLE IF NOT EXISTS character_vassals (
    liege_id BIGINT NOT NULL,
    vassal_id BIGINT NOT NULL,
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, liege_id, vassal_id),

    FOREIGN KEY (meta_id, liege_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, vassal_id) REFERENCES characters(meta_id, id)
);

-- character kill table init
CREATE TABLE IF NOT EXISTS character_kills (
    killer_id INTEGER NOT NULL,
    victim_id INTEGER NOT NULL,
    meta_id INTEGER NOT NULL REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, killer_id, victim_id),
    FOREIGN KEY (meta_id, killer_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, victim_id) REFERENCES characters(meta_id, id)
);

CREATE TABLE IF NOT EXISTS character_languages (
    character_id INTEGER NOT NULL,
    language TEXT NOT NULL,                   -- 语言（已本地化）
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, character_id, language),

    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id)
);

CREATE TABLE IF NOT EXISTS character_memories (
    character_id BIGINT NOT NULL,
    memory_id INTEGER NOT NULL,
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, character_id, memory_id),

    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, memory_id) REFERENCES memories(meta_id, id)
);

-- character artifact table init
CREATE TABLE IF NOT EXISTS character_artifacts (
    character_id INTEGER NOT NULL,
    artifact_id INTEGER NOT NULL,
    meta_id INTEGER REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, character_id, artifact_id),
    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, artifact_id) REFERENCES artifacts(meta_id, id)
);

-- title claim table init
CREATE TABLE IF NOT EXISTS title_claims (
    title_id INTEGER NOT NULL,
    claimant_id INTEGER NOT NULL,
    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID
    PRIMARY KEY (meta_id, title_id, claimant_id),
    FOREIGN KEY (meta_id, title_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, claimant_id) REFERENCES characters(meta_id, id)
);

-- title history table init
CREATE TABLE IF NOT EXISTS title_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title_id BIGINT NOT NULL REFERENCES titles(id),
    date DATE NOT NULL,                       -- 历史日期
    holder_id BIGINT REFERENCES characters(id), -- 持有者
    action TEXT NOT NULL,                     -- 历史事件（已本地化）
    
    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID

    FOREIGN KEY (meta_id, title_id) REFERENCES titles(meta_id, id),
    FOREIGN KEY (meta_id, holder_id) REFERENCES characters(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_title_history_date ON title_history(meta_id, date);

-- faith tenet table init
CREATE TABLE IF NOT EXISTS faith_tenets (
    faith_id INTEGER NOT NULL,
    tenet_name TEXT NOT NULL,                 -- 信条名称（已本地化）
    meta_id INTEGER NOT NULL REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, faith_id, tenet_name),
    FOREIGN KEY (meta_id, faith_id) REFERENCES faiths(meta_id, id)
);

CREATE TABLE IF NOT EXISTS faith_doctrines (
    faith_id INTEGER NOT NULL,
    doctrine_name TEXT NOT NULL,              -- 教义名称（已本地化）
    meta_id INTEGER NOT NULL REFERENCES game_metadata(id),
    PRIMARY KEY (meta_id, faith_id, doctrine_name),
    FOREIGN KEY (meta_id, faith_id) REFERENCES faiths(meta_id, id)
);

-- culture tradition table init
CREATE TABLE IF NOT EXISTS culture_traditions (
    culture_id INTEGER NOT NULL,
    tradition_name TEXT NOT NULL,             -- 传统名称（已本地化）
    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID
    PRIMARY KEY (meta_id, culture_id, tradition_name),
    FOREIGN KEY (meta_id, culture_id) REFERENCES cultures(meta_id, id)
);

--culture era table init
 CREATE TABLE IF NOT EXISTS culture_eras
 (
     culture_id INTEGER NOT NULL,
     info  TEXT  NOT NULL, -- 时代名称（已本地化）
     begin_era INTEGER,
     end_era INTEGER,
     meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID
     PRIMARY KEY (meta_id, culture_id, info),
     FOREIGN KEY (meta_id, culture_id) REFERENCES cultures (meta_id, id)
 );

--culture tree table init
CREATE TABLE IF NOT EXISTS culture_trees
(
    parent_culture_id INTEGER NOT NULL,
    child_culture_id INTEGER NOT NULL,
    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID
    PRIMARY KEY (meta_id, parent_culture_id, child_culture_id),
    FOREIGN KEY (meta_id, parent_culture_id) REFERENCES cultures (meta_id, id),
    FOREIGN KEY (meta_id, child_culture_id) REFERENCES cultures (meta_id, id)
);

--index
CREATE INDEX IF NOT EXISTS idx_culture_tress_parent ON culture_trees (meta_id, parent_culture_id, child_culture_id);

-- dynasty perk table init
CREATE TABLE IF NOT EXISTS dynasty_perks (
    dynasty_id BIGINT NOT NULL,
    perk_name TEXT NOT NULL,                  -- 特质名称（已本地化）
    level INTEGER NOT NULL DEFAULT 1,       -- 特质等级
    meta_id INTEGER REFERENCES game_metadata(id), -- 元数据ID

    PRIMARY KEY (meta_id, dynasty_id, perk_name),
    FOREIGN KEY (meta_id, dynasty_id) REFERENCES dynasties(meta_id, id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_dynasty_perks_dynasty ON dynasty_perks(meta_id, dynasty_id);

-- game metadata table init
CREATE TABLE IF NOT EXISTS game_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    current_date DATE,               -- 当前游戏日期
    offset_date DATE,                 -- 游戏开始日期
    save_file_name TEXT,                      -- 存档文件名
    language TEXT NOT NULL DEFAULT 'english', -- 游戏语言
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS players (
    id INTEGER,
    player_name TEXT NOT NULL,                -- 玩家名称
    curr_character_id INTEGER,
    meta_id INTEGER REFERENCES game_metadata(id),

    PRIMARY KEY (meta_id, player_name),

    FOREIGN KEY (meta_id, curr_character_id) REFERENCES characters(meta_id, id)
);

CREATE TABLE IF NOT EXISTS players_id_seq (
     meta_id  INTEGER PRIMARY KEY,
     last_id  INTEGER NOT NULL DEFAULT 0
);

-- player 序号触发器
CREATE TRIGGER IF NOT EXISTS trg_players_seq_ai
    AFTER INSERT ON players
    WHEN NEW.id IS NULL
BEGIN
    -- 初始化对应 meta 的游标
    INSERT INTO players_id_seq(meta_id, last_id)
    VALUES (NEW.meta_id, 0);

    -- 递增并取号
    UPDATE players_id_seq
    SET last_id = last_id + 1
    WHERE meta_id = NEW.meta_id;

    -- 写回当前行（通过 rowid 锁定刚插入的行）
    UPDATE players
    SET id = (SELECT last_id FROM players_id_seq WHERE meta_id = NEW.meta_id)
    WHERE rowid = NEW.rowid;
END;



CREATE TABLE IF NOT EXISTS lineages (
    id INTEGER,
    character_id INTEGER,
    player_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    score INTEGER NOT NULL,
    prestige INTEGER NOT NULL,
    piety INTEGER NOT NULL,
    dread INTEGER NOT NULL,
    lifestyle TEXT,
    meta_id INTEGER REFERENCES game_metadata(id),

    PRIMARY KEY (meta_id, id),

    FOREIGN KEY (meta_id, character_id) REFERENCES characters(meta_id, id),
    FOREIGN KEY (meta_id, player_id) REFERENCES players(meta_id, id)
);

CREATE TABLE IF NOT EXISTS lineages_id_seq (
    meta_id  INTEGER,
    player_id INTEGER,
    last_id  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (meta_id, player_id)
);

-- lineage 序号触发器
CREATE TRIGGER IF NOT EXISTS trg_lineages_seq_ai
    AFTER INSERT ON lineages
    WHEN NEW.id IS NULL
BEGIN
    -- 初始化对应 meta 的游标
    INSERT OR IGNORE INTO lineages_id_seq(meta_id, player_id, last_id)
    VALUES (NEW.meta_id, NEW.player_id, 0);

    -- 递增并取号
    UPDATE lineages_id_seq
    SET last_id = last_id + 1
    WHERE meta_id = NEW.meta_id AND player_id = NEW.player_id;

    -- 写回当前行（通过 rowid 锁定刚插入的行）
    UPDATE lineages
    SET id = (SELECT last_id FROM lineages_id_seq WHERE meta_id = NEW.meta_id AND player_id = NEW.player_id)
    WHERE rowid = NEW.rowid;
END;

CREATE TABLE IF NOT EXISTS artifacts_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    artifact_id INTEGER NOT NULL REFERENCES artifacts(id),
    info TEXT NOT NULL,               -- 历史信息（已本地化）
    date DATE NOT NULL,               -- 历史日期
    character_from_id BIGINT REFERENCES characters(id), -- 相关角色
    character_to_id BIGINT REFERENCES characters(id),   -- 相关角色
    meta_id INTEGER REFERENCES game_metadata(id) -- 元数据ID
);

-- index
CREATE INDEX IF NOT EXISTS idx_artifacts_history_meta_id ON artifacts_history(meta_id, artifact_id);