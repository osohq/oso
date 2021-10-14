import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions;

public class TestApp {
  public static void testPolicy() throws Exception {
    Oso oso = App.setupOso();

    App.Organization alphaAssociation = new App.Organization("Alpha Association");
    App.Organization betaBusiness = new App.Organization("Beta Business");

    App.Repository affineTypes = new App.Repository("Affine Types", alphaAssociation);
    App.Repository allocator = new App.Repository("Allocator", alphaAssociation);
    App.Repository bubbleSort = new App.Repository("Bubble Sort", betaBusiness);
    App.Repository benchmarks = new App.Repository("Benchmarks", betaBusiness);

    App.User ariana = new App.User("Ariana");
    App.User bhavik = new App.User("Bhavik");

    ariana.assignRoleForResource("owner", alphaAssociation);
    bhavik.assignRoleForResource("contributor", bubbleSort);
    bhavik.assignRoleForResource("maintainer", benchmarks);

    oso.authorize(ariana, "read", affineTypes);
    oso.authorize(ariana, "push", affineTypes);
    oso.authorize(ariana, "read", allocator);
    oso.authorize(ariana, "push", allocator);
    try { oso.authorize(ariana, "read", bubbleSort); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(ariana, "push", bubbleSort); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(ariana, "read", benchmarks); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(ariana, "push", benchmarks); } catch(Exceptions.NotFoundException e) {}

    try { oso.authorize(bhavik, "read", affineTypes); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(bhavik, "push", affineTypes); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(bhavik, "read", allocator); } catch(Exceptions.NotFoundException e) {}
    try { oso.authorize(bhavik, "push", allocator); } catch(Exceptions.NotFoundException e) {}
    oso.authorize(bhavik, "read", bubbleSort);
    try { oso.authorize(bhavik, "push", bubbleSort); } catch(Exceptions.ForbiddenException e) {}
    oso.authorize(bhavik, "read", benchmarks);
    oso.authorize(bhavik, "push", benchmarks);
  }

  public static void main(String[] args) throws Exception {
    testPolicy();
    System.out.println("Java RBAC tests pass!");
  }
}
