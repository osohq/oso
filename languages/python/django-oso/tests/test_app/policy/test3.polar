policy_load_test(3);

allow_scope(user, _, test_app::TestScope, new Q(user: user));
allow_scope(_, "GET", test_app::TestScope, new Q(public: true));
