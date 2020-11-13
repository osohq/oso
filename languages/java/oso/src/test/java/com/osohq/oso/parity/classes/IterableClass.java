package com.osohq.oso.parity.classes;

import java.util.*;

public class IterableClass implements Iterable<Integer> {
    private List<Integer> list;

    public IterableClass(List<Integer> list) {
        this.list = list;
    }

    // code for data structure
    public Integer sum() {
        int count = 0;
        for (int i : list) {
            count += i;
        }
        return count;
    }

    // code for data structure
    public Iterator<Integer> iterator() {
        return list.iterator();
    }
}