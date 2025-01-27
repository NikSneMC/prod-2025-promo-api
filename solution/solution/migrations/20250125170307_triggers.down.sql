DROP TRIGGER IF EXISTS comment_watcher ON promos;

DROP FUNCTION IF EXISTS update_comment_count;


DROP TRIGGER IF EXISTS like_watcher ON promos;

DROP FUNCTION IF EXISTS update_like_count;


DROP TRIGGER IF EXISTS active_watcher_activations ON activations;

DROP TRIGGER IF EXISTS active_watcher_promos ON promos;

DROP FUNCTION IF EXISTS check_promo_active_trigger;

DROP FUNCTION IF EXISTS update_promos_active;

DROP FUNCTION IF EXISTS update_promo_active;