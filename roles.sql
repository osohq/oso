begin;


-- test data schema
create table users (
	name text primary key
);

create table orgs (
	name text primary key
);

create table repos (
	name text primary key,
	org_name text
);

create table issues (
	name text primary key,
	repo_name text
);

insert into users values
	('leina'),
	('steve'),
	('gabe');

insert into orgs values
	('apple'),
	('osohq');

insert into repos values
	('ios', 'apple'),
	('oso', 'osohq'),
	('demo', 'osohq');

insert into issues values
	('laggy', 'ios'),
	('bug', 'oso');

-- each user needs their own postgres role
create role leina;
create role steve;
create role gabe;

-- Here's an oso policy
--  resource(_type: Org, "org", actions, roles) if
--      actions = [
--          "SELECT",
--          "UPDATE"
--      ] and
--      roles = {
--          member: {
--              permissions: ["SELECT"],
--              implies: ["repo:reader"]
--          },
--          owner: {
--              permissions: ["UPDATE"],
--              implies: ["member", "repo:writer"]
--          }
--      };
--  resource(_type: Repo, "repo", actions, roles) if
--      actions = [
--          "SELECT",
--          "UPDATE"
--      ] and
--      roles = {
--          writer: {
--              permissions: ["UPDATE", "issue:UPDATE"],
--              implies: ["reader"]
--          },
--          reader: {
--              permissions: ["SELECT", "issue:SELECT"]
--          }
--      };
--  resource(_type: Issue, "issue", actions, {}) if
--      actions = [
--          "SELECT",
--          "UPDATE"
--      ];
--  parent_child(parent_org: Org, repo: Repo) if
--      parent_org in ORGANIZATIONS and
--      repo.org_name = parent_org.name;
--  parent_child(parent_repo: Repo, issue: Issue) if
--      parent_repo in REPOSITORIES and
--      issue.repo_name = parent_repo.name;

-- Then there's some really good code that turns that oso policy
-- into this sql.

-- BEGIN GENERATED BASED ON POLICY

-- oso internal schema
create table org_role_assignments (
	user_id text,
	org_id text,
	role text
);
create table repo_role_assignments (
	user_id text,
	repo_id text,
	role text
);
create table issue_role_assignments (
	user_id text,
	issue_id text,
	role text
);

GRANT SELECT ON org_role_assignments TO public;
GRANT SELECT ON repo_role_assignments TO public;
GRANT SELECT ON issue_role_assignments TO public;

GRANT SELECT ON orgs TO public;
ALTER TABLE orgs ENABLE ROW LEVEL SECURITY;
GRANT SELECT ON repos TO public;
ALTER TABLE repos ENABLE ROW LEVEL SECURITY;
GRANT SELECT ON issues TO public;
ALTER TABLE issues ENABLE ROW LEVEL SECURITY;


-- This is all pre-computed. There will be a policy
-- for every action-table combination.
-- We just get the set of roles that are correct and
-- the policy just checks if they're assiged.
CREATE POLICY oso_orgs_select ON orgs FOR SELECT
	-- From the policy
	-- SELECT orgs is on org:member
	-- org:owner implies org:member
	-- so if user is member or owner for org, they can select
	USING (
		EXISTS (
			select *
			from org_role_assignments ora where
			ora.user_id = current_user and
			ora.org_id = name and
			ora.role = 'owner'	
		) or
		EXISTS (
			select *
			from org_role_assignments ora where
			ora.user_id = current_user and
			ora.org_id = name and
			ora.role = 'member'
		)
	);

CREATE POLICY oso_orgs_update ON orgs FOR UPDATE
	USING (
		EXISTS (
			select *
			from org_role_assignments ora where
			ora.user_id = current_user and
			ora.org_id = name and
			ora.role = 'owner'
		)
	);

CREATE POLICY oso_repos_select ON repos FOR SELECT
	-- repo:reader
	-- repo:writer
	-- org:member
	-- org:owner
	USING (
		EXISTS (
			select *
			from repo_role_assignments rra
			where
			rra.user_id = current_user and
			rra.repo_id = name and
			rra.role = 'reader'	
		) or
		EXISTS (
			select *
			from repo_role_assignments rra
			where
			rra.user_id = current_user and
			rra.repo_id = name and
			rra.role = 'writer'
		) or
		EXISTS (
			select *
			from org_role_assignments ora
			where
			ora.user_id = current_user and
			ora.org_id = org_name and
			ora.role = 'owner'
		) or
		EXISTS (
			select *
			from org_role_assignments ora
			where
			ora.user_id = current_user and
			ora.org_id = org_name and
			ora.role = 'member'
		)
	);

CREATE POLICY oso_issues_select ON issues FOR SELECT
	-- repo:reader
	-- repo:writer
	-- org:member
	-- org:owner
	USING (
		EXISTS (
			select *
			from repo_role_assignments rra
			where
			rra.user_id = current_user and
			rra.repo_id = repo_name and
			rra.role = 'reader'	
		) or
		EXISTS (
			select *
			from repo_role_assignments rra
			where
			rra.user_id = current_user and
			rra.repo_id = repo_name and
			rra.role = 'writer'
		) or
		EXISTS (
			select *
			from repos r
			join orgs o
			on o.name = r.org_name
			join org_role_assignments ora
			on ora.org_id = o.name
			where
			ora.user_id = current_user and
			ora.org_id = o.name and
			r.name = repo_name and
			ora.role = 'owner'
		) or
		EXISTS (
			select *
			from repos r
			join orgs o
			on o.name = r.org_name
			join org_role_assignments ora
			on ora.org_id = o.name
			where
			ora.user_id = current_user and
			ora.org_id = o.name and
			r.name = repo_name and
			ora.role = 'member'
		)
	);



-- END GENERATED BASED ON POLICY

-- -- assign some oso roles to test it
insert into org_role_assignments values
	('leina', 'osohq', 'owner'),
 	('steve', 'osohq', 'member');

-- 			select *
-- 			from org_role_assignments ora
-- 			where ora.user_id = 'leina' and
-- 			ora.org_id = 'osohq' and
-- 			ora.role = 'owner';

			--where
			--ora.user_name = 'leina' and
			--ora.org_name = 'osohq' and
			--ora.role = 'member';

-- set role leina;
--select * from orgs;
--select * from repos;
--select * from issues;

set role leina;
select * from issues;


rollback;