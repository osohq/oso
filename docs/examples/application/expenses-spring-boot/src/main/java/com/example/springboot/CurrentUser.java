package com.example.springboot;

import org.springframework.stereotype.Component;

@Component
public class CurrentUser {
    private Object user;

    public void set(User user) {
        this.user = user;
    }

    public void set(Guest guest) {
        this.user = guest;
    }

    public Object get() {
        return this.user;
    }

}