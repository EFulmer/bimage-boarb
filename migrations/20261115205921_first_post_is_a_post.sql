insert into posts (thread_id, created_at, active, image_id, body)
select id, created_at, active, image_id, body
from threads;

alter table threads drop column image_id;
alter table threads drop column body;
