package dev.br0b.mclootfinder.cli;

import java.util.HashMap;
import java.util.Map;

final class Arguments {
    private final Map<String, String> values = new HashMap<>();

    Arguments(String[] args, int startIndex) {
        for (int index = startIndex; index < args.length; index++) {
            String key = args[index];
            if (!key.startsWith("--")) {
                throw new IllegalArgumentException("Expected an option, got: " + key);
            }
            if (index + 1 >= args.length || args[index + 1].startsWith("--")) {
                values.put(key.substring(2), "true");
            } else {
                values.put(key.substring(2), args[++index]);
            }
        }
    }

    String text(String key, String fallback) {
        return values.getOrDefault(key, fallback);
    }

    long longValue(String key) {
        String value = values.get(key);
        if (value == null) {
            throw new IllegalArgumentException("Missing required option --" + key);
        }
        return Long.parseLong(value);
    }

    int intValue(String key, int fallback) {
        return Integer.parseInt(values.getOrDefault(key, Integer.toString(fallback)));
    }

    boolean flag(String key) {
        return Boolean.parseBoolean(values.getOrDefault(key, "false"));
    }
}

