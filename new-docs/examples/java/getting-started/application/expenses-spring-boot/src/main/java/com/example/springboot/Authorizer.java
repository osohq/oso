package com.example.springboot;

import com.osohq.oso.Exceptions.OsoException;
import com.osohq.oso.Oso;
import java.sql.SQLException;
import javax.annotation.Resource;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;
import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Component;
import org.springframework.web.server.ResponseStatusException;
import org.springframework.web.servlet.handler.HandlerInterceptorAdapter;

@Component
public class Authorizer extends HandlerInterceptorAdapter {
  @Resource(name = "setupOso")
  private Oso oso;

  @Resource(name = "requestScopeCurrentUser")
  private CurrentUser currentUser;

  @Override
  public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
      throws Exception {
    try {
      setCurrentUser(request);

      // Authorize the incoming request
      if (!oso.isAllowed(currentUser.get(), request.getMethod(), request)) {
        throw new ResponseStatusException(HttpStatus.FORBIDDEN, "oso authorization: unauthorized");
      }
    } catch (SQLException e) {
      throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "User not found", e);
    }
    return true;
  }

  /** Set current user from authorization header */
  private void setCurrentUser(HttpServletRequest request) throws SQLException {
    String email = request.getHeader("user");
    if (email == null) {
      currentUser.set(new Guest());
    } else {
      currentUser.set(User.lookup(email));
    }
  }

  /** oso authorization helper */
  public Object authorize(String action, Object resource) {
    try {
      if (!oso.isAllowed(currentUser.get(), action, resource)) {
        throw new ResponseStatusException(HttpStatus.FORBIDDEN, "oso authorization");
      }
    } catch (OsoException e) {
      throw new ResponseStatusException(HttpStatus.INTERNAL_SERVER_ERROR, null, e);
    }
    return resource;
  }
}
