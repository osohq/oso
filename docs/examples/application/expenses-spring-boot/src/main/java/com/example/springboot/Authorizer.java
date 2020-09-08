package com.example.springboot;

import java.sql.SQLException;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;

import javax.annotation.Resource;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;

import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Component;
import org.springframework.web.server.ResponseStatusException;
import org.springframework.web.servlet.handler.HandlerInterceptorAdapter;

import com.osohq.oso.Http;
import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.OsoException;

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
            Http http = new Http(request.getServerName(), request.getServletPath().toString(), getQuery(request));
            if (!oso.isAllowed(currentUser.get(), request.getMethod(), http)) {
                throw new ResponseStatusException(HttpStatus.FORBIDDEN, "oso authorization: unauthorized");
            }
        } catch (SQLException e) {
            throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "User not found", e);
        }
        return true;
    }

    /**
     * Get query from request parameters
     */
    private Map<String, String> getQuery(HttpServletRequest request) {
        return request.getParameterMap().entrySet().stream()
                .collect(Collectors.toMap(Entry::getKey, e -> e.getValue()[0]));
    }

    /**
     * Set current user from authorization header
     */
    private void setCurrentUser(HttpServletRequest request) throws SQLException {
        String email = request.getHeader("user");
        if (email == null) {
            currentUser.set(new Guest());
        } else {
            currentUser.set(User.lookup(email));
        }
    }

    /**
     * oso authorization helper
     */
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
