-- Add migration script here
ALTER TABLE items
DROP CONSTRAINT IF EXISTS items_category_check;

ALTER TABLE items
ADD CONSTRAINT items_category_check
CHECK (category IN ('food', 'tool', 'armor', 'weapon', 'materials'));