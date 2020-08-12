package com.example.springboot;

import java.io.IOException;

import javax.servlet.http.HttpServletRequest;

import org.springframework.beans.factory.annotation.Autowired;
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
        oso.registerClass(User.class, (m) -> new User(1, 2, 3, 4, "email", "title"), "User");
        oso.registerClass(Expense.class, (m) -> new Expense(1, "description", "submittedBy"), "Expense");
        oso.loadFile("src/main/oso/policy.polar");
        return oso;
    }
}