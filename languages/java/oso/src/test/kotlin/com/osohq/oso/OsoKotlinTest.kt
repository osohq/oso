package com.osohq.oso

import com.osohq.oso.Exceptions.OsoException
import org.junit.jupiter.api.Assertions
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import java.util.Set

class OsoKotlinTest {
    var o: Oso? = null

    data class User(val name: String) {
        fun companies(): List<Company> {
            return listOf(Company(1))
        }
    }

    data class Widget(val id: Int)
    data class Company(val id: Int) {
        fun role(a: User): String {
            return if (a.name == "president") {
                "admin"
            } else "guest"
        }
    }

    @BeforeEach
    fun setUp() {
        val testOso = javaClass.classLoader.getResource("test_oso.polar")
        o = Oso().apply {
            registerClass(User::class.java, "User")
            registerClass(Widget::class.java, "Widget")
            registerClass(Company::class.java, "Company")
            loadFile(testOso.path)
        }
    }

    @Test
    fun testIsAllowed() {
        val guest = User("guest")
        val resource1 = Widget(1)
        Assertions.assertTrue(o!!.isAllowed(guest, "get", resource1))
        val president = User("president")
        val company = Company(1)
        Assertions.assertTrue(o!!.isAllowed(president, "create", company))
    }

    @Test
    fun testFail() {
        val guest = User("guest")
        val widget = Widget(1)
        Assertions.assertFalse(o!!.isAllowed(guest, "not_allowed", widget))
    }

    @Test
    fun testInstanceFromExternalCall() {
        val company = Company(1)
        val guest = User("guest")
        Assertions.assertTrue(o!!.isAllowed(guest, "frob", company))

        // if the guest user can do it, then the dict should
        // create an instance of the user and be allowed
        val userMap = HashMap<String, String>()
        userMap["username"] = "guest"
        Assertions.assertTrue(o!!.isAllowed(userMap, "frob", company))
    }

    @Test
    fun testAllowModel() {
        val auditor = User("auditor")
        Assertions.assertTrue(o!!.isAllowed(auditor, "list", Company::class.java))
        Assertions.assertFalse(o!!.isAllowed(auditor, "list", Widget::class.java))
    }

    @Test
    fun testGetAllowedActions() {
        val o = Oso()
        o.registerClass(User::class.java, "User")
        o.registerClass(Widget::class.java, "Widget")
        o.loadStr("allow(_actor: User{name: \"sally\"}, action, _resource: Widget{id: 1})"
                + " if action in [\"CREATE\", \"READ\"];")
        val actor = User("sally")
        val widget = Widget(1)
        val actions = o.getAllowedActions(actor, widget)
        Assertions.assertEquals(actions.size, 2)
        Assertions.assertTrue(actions.contains("CREATE"))
        Assertions.assertTrue(actions.contains("READ"))
        o.clearRules()
        o.loadStr("allow(_actor: User{name: \"fred\"}, action, _resource: Widget{id: 2})"
                + " if action in [1, 2, 3, 4];")
        val actor2 = User("fred")
        val widget2 = Widget(2)
        val actions2 = o.getAllowedActions(actor2, widget2)
        Assertions.assertEquals(actions2.size, 4)
        Assertions.assertTrue(actions2.contains(1))
        Assertions.assertTrue(actions2.contains(2))
        Assertions.assertTrue(actions2.contains(3))
        Assertions.assertTrue(actions2.contains(4))
        val actor3 = User("doug")
        val widget3 = Widget(4)
        Assertions.assertTrue(o.getAllowedActions(actor3, widget3).isEmpty())
    }

    @Test
    fun testGetAllowedActionsWildcard() {
        val o = Oso()
        o.registerClass(User::class.java, "User")
        o.registerClass(Widget::class.java, "Widget")
        o.loadStr("allow(_actor: User{name: \"John\"}, _action, _resource: Widget{id: 1});")
        val actor = User("John")
        val widget = Widget(1)
        Assertions.assertEquals(Set.of("*"), o.getAllowedActions(actor, widget, true))
        Assertions.assertThrows(OsoException::class.java) { o.getAllowedActions(actor, widget, false) }
    }

    @Test
    fun testNotEqualOperator() {
        val oso = Oso()
        oso.registerClass(User::class.java, "User")
        oso.loadStr("allow(actor: User, _action, _resource) if actor != nil;")
        Assertions.assertFalse(oso.isAllowed(null, "foo", "foo"))
    }
}