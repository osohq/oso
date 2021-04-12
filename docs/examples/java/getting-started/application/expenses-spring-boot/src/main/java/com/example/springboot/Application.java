package com.example.springboot;

import com.osohq.oso.Exceptions;
import com.osohq.oso.Oso;
import java.io.IOException;
import javax.servlet.http.HttpServletRequest;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;
import org.springframework.web.context.annotation.RequestScope;

@SpringBootApplication
public class Application {

  public static void main(String[] args) {
    SpringApplication.run(Application.class, args);
  }

  @Bean
  @RequestScope
  public CurrentUser requestScopeCurrentUser() {
    return new CurrentUser();
  }

  @Bean
  public Oso setupOso() throws IOException, Exceptions.OsoException {
    Oso oso = new Oso();
    oso.registerClass(User.class, "User");
    oso.registerClass(Expense.class, "Expense");
    oso.registerClass(Organization.class, "Organization");
    oso.registerClass(HttpServletRequest.class, "Request");
    oso.loadFile("src/main/oso/authorization.polar");
    return oso;
  }
}
