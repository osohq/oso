[oso][trace]   Query(allow(<__main__.User object at 0x7f0013c99640> TYPE `User`, "view", <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`))
[oso][info]   QUERY RULE: allow(<__main__.User object at 0x7f0013c99640> TYPE `User`, "view", <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`), BINDINGS: {}
[oso][info]     APPLICABLE_RULES:
[oso][info]       allow(actor, action, resource) at line 11, column 5
[oso][trace]     TraceRule(...)
[oso][trace]     RULE: allow(actor, action, resource) if has_permission(actor, action, resource);
[oso][trace]     TraceStackPush
[oso][trace]     Unify(<__main__.User object at 0x7f0013c99640> TYPE `User`, _actor_4)
[oso][trace]     ⇒ bind: _actor_4 ← <__main__.User object at 0x7f0013c99640> TYPE `User`
[oso][trace]     Unify("view", _action_5)
[oso][trace]     ⇒ bind: _action_5 ← "view"
[oso][trace]     Unify(<__main__.Repository object at 0x7f0013c99610> TYPE `Repository`, _resource_6)
[oso][trace]     ⇒ bind: _resource_6 ← <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`
[oso][trace]     Query(has_permission(_actor_4, _action_5, _resource_6))
[oso][trace]       TraceStackPush
[oso][trace]       Query(has_permission(_actor_4, _action_5, _resource_6))
[oso][info]       QUERY RULE: has_permission(_actor_4, _action_5, _resource_6), BINDINGS: {_actor_4 => <__main__.User object at 0x7f0013c99640> TYPE `User`, _action_5 => "view", _resource_6 => <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`}
[oso][trace]         APPLICABLE_RULES:
[oso][trace]           has_permission(actor: Actor{}, "view", repository: Repository{}) at line 7, column 8
[oso][trace]         TraceRule(...)
[oso][trace]         RULE: has_permission(actor: Actor{}, "view", repository: Repository{}) if has_relation(actor, "owner", repository);
[oso][trace]         TraceStackPush
[oso][trace]         Unify(_actor_4, _actor_11)
[oso][trace]         ⇒ bind: _actor_4 ← _actor_11
[oso][trace]         Isa(_actor_11, Actor{})
[oso][trace]         MATCHES: _actor_11 matches Actor{}, BINDINGS: {_actor_11 => <__main__.User object at 0x7f0013c99640> TYPE `User`}
[oso][trace]         Isa(_actor_11, User{})
[oso][trace]         MATCHES: _actor_11 matches User{}, BINDINGS: {_actor_11 => <__main__.User object at 0x7f0013c99640> TYPE `User`}
[oso][trace]         Isa(<__main__.User object at 0x7f0013c99640> TYPE `User`, User{})
[oso][trace]         MATCHES: <__main__.User object at 0x7f0013c99640> TYPE `User` matches User{}, BINDINGS: {}
[oso][trace]         IsaExternal { instance: Term { source_info: Ffi, value: ExternalInstance(ExternalInstance { instance_id: 14, constructor: None, repr: None, class_repr: Some("User") }) }, literal: InstanceLiteral { tag: Symbol("User"), fields: Dictionary { fields: {} } } }
[oso][trace]         ⇒ bind: _isa_13 ← false
[oso][trace]         Unify(_isa_13, true)
[oso][trace]         Unify(true, true)
[oso][trace]         Isa(<__main__.User object at 0x7f0013c99640> TYPE `User`, {})
[oso][trace]         MATCHES: <__main__.User object at 0x7f0013c99640> TYPE `User` matches {}, BINDINGS: {}
[oso][trace]         Unify(_action_5, "view")
[oso][trace]         Unify("view", "view")
[oso][trace]         Unify(_resource_6, _repository_12)
[oso][trace]         ⇒ bind: _resource_6 ← _repository_12
[oso][trace]         Isa(_repository_12, Repository{})
[oso][trace]         MATCHES: _repository_12 matches Repository{}, BINDINGS: {_repository_12 => <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`}
[oso][trace]         Isa(<__main__.Repository object at 0x7f0013c99610> TYPE `Repository`, Repository{})
[oso][trace]         MATCHES: <__main__.Repository object at 0x7f0013c99610> TYPE `Repository` matches Repository{}, BINDINGS: {}
[oso][trace]         IsaExternal { instance: Term { source_info: Ffi, value: ExternalInstance(ExternalInstance { instance_id: 15, constructor: None, repr: None, class_repr: Some("Repository") }) }, literal: InstanceLiteral { tag: Symbol("Repository"), fields: Dictionary { fields: {} } } }
[oso][trace]         ⇒ bind: _isa_14 ← false
[oso][trace]         Unify(_isa_14, true)
[oso][trace]         Unify(true, true)
[oso][trace]         Isa(<__main__.Repository object at 0x7f0013c99610> TYPE `Repository`, {})
[oso][trace]         MATCHES: <__main__.Repository object at 0x7f0013c99610> TYPE `Repository` matches {}, BINDINGS: {}
[oso][trace]         Query(has_relation(_actor_11, "owner", _repository_12))
[oso][trace]           TraceStackPush
[oso][trace]           Query(has_relation(_actor_11, "owner", _repository_12))
[oso][info]           QUERY RULE: has_relation(_actor_11, "owner", _repository_12), BINDINGS: {_actor_11 => <__main__.User object at 0x7f0013c99640> TYPE `User`, _repository_12 => <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`}
[oso][trace]             APPLICABLE_RULES:
[oso][trace]               has_relation(_user: User{}, "owner", _repository: Repository{}) at line 10, column 5
[oso][trace]             TraceRule(...)
[oso][trace]             RULE: has_relation(_user: User{}, "owner", _repository: Repository{});
[oso][trace]             TraceStackPush
[oso][trace]             Unify(_actor_11, __user_19)
[oso][trace]             ⇒ bind: _actor_11 ← __user_19
[oso][trace]             Isa(__user_19, User{})
[oso][trace]             MATCHES: __user_19 matches User{}, BINDINGS: {__user_19 => <__main__.User object at 0x7f0013c99640> TYPE `User`}
[oso][trace]             Isa(<__main__.User object at 0x7f0013c99640> TYPE `User`, User{})
[oso][trace]             MATCHES: <__main__.User object at 0x7f0013c99640> TYPE `User` matches User{}, BINDINGS: {}
[oso][trace]             IsaExternal { instance: Term { source_info: Ffi, value: ExternalInstance(ExternalInstance { instance_id: 14, constructor: None, repr: None, class_repr: Some("User") }) }, literal: InstanceLiteral { tag: Symbol("User"), fields: Dictionary { fields: {} } } }
[oso][trace]             ⇒ bind: _isa_21 ← false
[oso][trace]             Unify(_isa_21, true)
[oso][trace]             Unify(true, true)
[oso][trace]             Isa(<__main__.User object at 0x7f0013c99640> TYPE `User`, {})
[oso][trace]             MATCHES: <__main__.User object at 0x7f0013c99640> TYPE `User` matches {}, BINDINGS: {}
[oso][trace]             Unify("owner", "owner")
[oso][trace]             Unify(_repository_12, __repository_20)
[oso][trace]             ⇒ bind: _repository_12 ← __repository_20
[oso][trace]             Isa(__repository_20, Repository{})
[oso][trace]             MATCHES: __repository_20 matches Repository{}, BINDINGS: {__repository_20 => <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`}
[oso][trace]             Isa(<__main__.Repository object at 0x7f0013c99610> TYPE `Repository`, Repository{})
[oso][trace]             MATCHES: <__main__.Repository object at 0x7f0013c99610> TYPE `Repository` matches Repository{}, BINDINGS: {}
[oso][trace]             IsaExternal { instance: Term { source_info: Ffi, value: ExternalInstance(ExternalInstance { instance_id: 15, constructor: None, repr: None, class_repr: Some("Repository") }) }, literal: InstanceLiteral { tag: Symbol("Repository"), fields: Dictionary { fields: {} } } }
[oso][trace]             ⇒ bind: _isa_22 ← false
[oso][trace]             Unify(_isa_22, true)
[oso][trace]             Unify(true, true)
[oso][trace]             Isa(<__main__.Repository object at 0x7f0013c99610> TYPE `Repository`, {})
[oso][trace]             MATCHES: <__main__.Repository object at 0x7f0013c99610> TYPE `Repository` matches {}, BINDINGS: {}
[oso][trace]             Query((true))
[oso][trace]               TraceStackPush
[oso][trace]               TraceStackPop
[oso][trace]               PopQuery((true))
[oso][trace]             TraceStackPop
[oso][trace]             TraceStackPop
[oso][trace]             PopQuery(has_relation(_actor_11, "owner", _repository_12))
[oso][trace]           TraceStackPop
[oso][trace]           PopQuery(has_relation(_actor_11, "owner", _repository_12))
[oso][trace]         TraceStackPop
[oso][trace]         TraceStackPop
[oso][trace]         PopQuery(has_permission(_actor_4, _action_5, _resource_6))
[oso][trace]       TraceStackPop
[oso][trace]       PopQuery(has_permission(_actor_4, _action_5, _resource_6))
[oso][trace]     TraceStackPop
[oso][trace]     TraceStackPop
[oso][trace]     PopQuery(allow(<__main__.User object at 0x7f0013c99640> TYPE `User`, "view", <__main__.Repository object at 0x7f0013c99610> TYPE `Repository`))
[oso][info]   RESULT: SUCCESS
True
[oso][trace]   Query(allow(<__main__.User object at 0x7f0013c99640> TYPE `User`, "view", <__main__.Project object at 0x7f0013c99310> TYPE `UNKNOWN`))
[oso][info]   QUERY RULE: allow(<__main__.User object at 0x7f0013c99640> TYPE `User`, "view", <__main__.Project object at 0x7f0013c99310> TYPE `UNKNOWN`), BINDINGS: {}
[oso][info]     APPLICABLE_RULES:
[oso][info]       allow(actor, action, resource) at line 11, column 5
[oso][trace]     TraceRule(...)
[oso][trace]     RULE: allow(actor, action, resource) if has_permission(actor, action, resource);
[oso][trace]     TraceStackPush
[oso][trace]     Unify(<__main__.User object at 0x7f0013c99640> TYPE `User`, _actor_26)
[oso][trace]     ⇒ bind: _actor_26 ← <__main__.User object at 0x7f0013c99640> TYPE `User`
[oso][trace]     Unify("view", _action_27)
[oso][trace]     ⇒ bind: _action_27 ← "view"
[oso][trace]     Unify(<__main__.Project object at 0x7f0013c99310> TYPE `UNKNOWN`, _resource_28)
[oso][trace]     ⇒ bind: _resource_28 ← <__main__.Project object at 0x7f0013c99310> TYPE `UNKNOWN`
[oso][trace]     Query(has_permission(_actor_26, _action_27, _resource_28))
[oso][trace]       TraceStackPush
[oso][trace]       Query(has_permission(_actor_26, _action_27, _resource_28))
[oso][info]       QUERY RULE: has_permission(_actor_26, _action_27, _resource_28), BINDINGS: {_action_27 => "view", _actor_26 => <__main__.User object at 0x7f0013c99640> TYPE `User`, _resource_28 => <__main__.Project object at 0x7f0013c99310> TYPE `UNKNOWN`}
[oso][info]         No matching rules found
False


