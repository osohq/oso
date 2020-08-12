package com.example.springboot;

import java.io.IOException;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;

import com.osohq.oso.Exceptions;
import com.osohq.oso.Oso;

@SpringBootApplication
public class Application {

    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }

    @Bean
    public Oso setupOso() throws IOException, Exceptions.OsoException {
        Oso oso = new Oso();
        oso.registerClass(User.class, "User");
        oso.registerClass(Expense.class, "Expense");
        oso.registerClass(Organization.class, "Organization");
        oso.loadFile("src/main/oso/policy.polar");
        return oso;
    }
}