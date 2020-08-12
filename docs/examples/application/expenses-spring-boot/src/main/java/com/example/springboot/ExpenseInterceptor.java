package com.example.springboot;

import java.io.IOException;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;

import javax.annotation.Resource;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;

import org.springframework.web.servlet.handler.HandlerInterceptorAdapter;

import com.osohq.oso.Http;
import com.osohq.oso.Oso;
import com.osohq.oso.Exceptions.OsoException;

public class ExpenseInterceptor extends HandlerInterceptorAdapter {
    @Resource(name = "setupOso")
    private Oso oso;

    @Override
    public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
            throws Exception {

        Map<String, String> query = request.getParameterMap().entrySet().stream()
                .collect(Collectors.toMap(Entry::getKey, e -> e.getValue()[0]));

        User user = User.lookup(request.getHeader("user"));
        if (oso.isAllowed(user, request.getMethod(),
                new Http(request.getServerName(), request.getRequestURL().toString(), query))) {
            return true;
        } else {
            response.getWriter().write("Forbidden");
            return false;
        }
    }
}