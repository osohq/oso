package com.example.springboot;

import java.sql.SQLException;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;

import javax.annotation.Resource;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;

import org.springframework.stereotype.Component;
import org.springframework.web.servlet.handler.HandlerInterceptorAdapter;

import com.osohq.oso.Http;
import com.osohq.oso.Oso;

@Component
public class OsoInterceptor extends HandlerInterceptorAdapter {
    @Resource(name = "setupOso")
    private Oso oso;

    @Override
    public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
            throws Exception {

        Map<String, String> query = request.getParameterMap().entrySet().stream()
                .collect(Collectors.toMap(Entry::getKey, e -> e.getValue()[0]));

        try {

            User user = User.lookup(request.getHeader("user"));
            Http http = new Http(request.getServerName(), request.getServletPath().toString(), query);
            if (oso.isAllowed(user, request.getMethod(), http)) {
                return true;
            } else {
                response.getWriter().write("Forbidden");
                return false;
            }
        } catch (SQLException e) {
            response.getWriter().write("Forbidden");
            return false;
        }
    }
}