MACRO_POSTGRES_PORT := env_var_or_default("MACRO_POSTGRES_PORT", "5433")
DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://user:password@localhost:" + MACRO_POSTGRES_PORT + "/macrodb")
