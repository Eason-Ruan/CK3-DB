-- 先删除有外键依赖的表
DROP TABLE IF EXISTS artifacts_history;
DROP TABLE IF EXISTS localization_keys;
DROP TABLE IF EXISTS dynasty_perks;
DROP TABLE IF EXISTS culture_trees;
DROP TABLE IF EXISTS culture_eras;
DROP TABLE IF EXISTS culture_traditions;
DROP TABLE IF EXISTS faith_doctrines;
DROP TABLE IF EXISTS faith_tenets;
DROP TABLE IF EXISTS title_history;
DROP TABLE IF EXISTS title_claims;
DROP TABLE IF EXISTS character_kills;
DROP TABLE IF EXISTS character_relationships;
DROP TABLE IF EXISTS character_titles;
DROP TABLE IF EXISTS character_traits;
DROP TABLE IF EXISTS memory_variables;
-- 删除主表
DROP TABLE IF EXISTS characters;
DROP TABLE IF EXISTS memories;
DROP TABLE IF EXISTS artifacts;
DROP TABLE IF EXISTS houses;
DROP TABLE IF EXISTS dynasties;
DROP TABLE IF EXISTS cultures;
DROP TABLE IF EXISTS faiths;
DROP TABLE IF EXISTS titles;
DROP TABLE IF EXISTS game_metadata;
DROP TABLE IF EXISTS players;
DROP TABLE IF EXISTS lineages;
