CREATE OR REPLACE FUNCTION update_promo_active(pr promos) RETURNS void AS
$$
DECLARE
    timestamp      timestamptz := now();
    is_active      bool        := true;
    current_active bool;
BEGIN
    is_active := is_active AND (pr.active_from <= timestamp AND timestamp <= pr.active_until);

    IF pr.mode = 'COMMON' THEN
        is_active := is_active AND (pr.used_count < pr.max_count);
    ELSIF pr.mode = 'UNIQUE' THEN
        is_active := is_active AND (pr.used_count < array_length(pr.promo_unique, 1));
    END IF;

    SELECT promos.active
    INTO current_active
    FROM promos
    WHERE id = pr.id;

    IF current_active IS DISTINCT FROM is_active THEN
        UPDATE promos
        SET active = is_active
        WHERE id = pr.id;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_promos_active() RETURNS void AS
$$
DECLARE
    pr promos;
BEGIN
    FOR pr IN SELECT * FROM promos LOOP
        PERFORM update_promo_active(pr);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_promo_active_trigger()
    RETURNS TRIGGER AS
$$
DECLARE
    pr promos;
BEGIN
    IF TG_TABLE_NAME = 'promos' THEN
        IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
            pr := NEW;
        ELSIF TG_OP = 'DELETE' THEN
            pr := OLD;
        END IF;
    ELSIF TG_TABLE_NAME = 'activations' THEN
        SELECT *
        INTO pr
        FROM promos
        WHERE id = NEW.promo_id;
    END IF;

    PERFORM update_promo_active(pr);

    RETURN NULL;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER active_watcher_promos
    AFTER INSERT OR UPDATE OR DELETE
    ON promos
    FOR EACH ROW
EXECUTE FUNCTION update_promo_active_trigger();

CREATE TRIGGER active_watcher_activations
    AFTER INSERT
    ON activations
    FOR EACH ROW
EXECUTE FUNCTION update_promo_active_trigger();


CREATE OR REPLACE FUNCTION update_like_count()
    RETURNS TRIGGER AS
$$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE promos
        SET like_count = like_count + 1
        WHERE id = NEW.promo_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE promos
        SET like_count = like_count - 1
        WHERE id = OLD.promo_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER like_watcher
    AFTER INSERT OR DELETE
    ON likes
    FOR EACH ROW
EXECUTE FUNCTION update_like_count();


CREATE OR REPLACE FUNCTION update_comment_count()
    RETURNS TRIGGER AS
$$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE promos
        SET comment_count = comment_count + 1
        WHERE id = NEW.promo_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE promos
        SET comment_count = comment_count - 1
        WHERE id = OLD.promo_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER comment_watcher
    AFTER INSERT OR DELETE
    ON comments
    FOR EACH ROW
EXECUTE FUNCTION update_comment_count();