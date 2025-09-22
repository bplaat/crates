/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! This is a simple server that listens on a Unix domain socket for javac commands,
//! executes them, and returns the result. This is handy because the Java compiler which
//! is written in Java is kept running in a single hot JVM, which makes it much faster.

import java.io.ByteArrayOutputStream;
import java.net.StandardProtocolFamily;
import java.net.UnixDomainSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.ServerSocketChannel;
import java.nio.channels.SocketChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicLong;
import javax.tools.ToolProvider;

public class JavacServer {
    private static final String SOCKET_PATH = "/tmp/.bob/javac";
    private static final long TIMEOUT_MS = 10 * 60 * 1000; // 10 minutes

    public static void main(String[] args) throws Exception {
        // Check if server is already running
        var socketPath = Paths.get(SOCKET_PATH);
        Files.createDirectories(socketPath.getParent());
        if (Files.exists(socketPath)) {
            System.out.println("Server is already running");
            System.exit(1);
        }

        // Clean up socket file on exit
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            try {
                Files.deleteIfExists(socketPath);
            } catch (Exception e) {
                e.printStackTrace();
                System.exit(1);
            }
        }));

        // Start listening on unix socket
        var serverChannel = ServerSocketChannel.open(StandardProtocolFamily.UNIX);
        serverChannel.bind(UnixDomainSocketAddress.of(socketPath));
        System.out.println("Server listening at: " + SOCKET_PATH);

        // Close server after inactivity
        var lastMessageTime = new AtomicLong(System.currentTimeMillis());
        new Thread(() -> {
            try {
                for (;;) {
                    var currentTime = System.currentTimeMillis();
                    if (currentTime - lastMessageTime.get() > TIMEOUT_MS) {
                        System.out.println("No message received for 10 minutes, shutting down...");
                        serverChannel.close();
                        System.exit(0);
                    }
                    Thread.sleep(1000);
                }
            } catch (Exception e) {
                e.printStackTrace();
                System.exit(1);
            }
        }).start();

        // Listen for incoming messages
        var compiler = ToolProvider.getSystemJavaCompiler();
        for (;;) {
            var channel = serverChannel.accept();
            new Thread(() -> {
                try {
                    // Read input from SocketChannel into a fixed 4096 byte buffer
                    var buffer = ByteBuffer.allocate(4096);
                    if (channel.read(buffer) <= 0) {
                        channel.close();
                        return;
                    }
                    buffer.flip();
                    var line = StandardCharsets.UTF_8.decode(buffer).toString().trim();
                    if (line.isEmpty()) {
                        channel.close();
                        return;
                    }

                    // Update last message time
                    System.out.println(line);
                    lastMessageTime.set(System.currentTimeMillis());

                    // Parse command line
                    var argList = parseCommand(line);
                    if (argList.isEmpty() || !argList.get(0).equals("javac")) {
                        channel.close();
                        return;
                    }
                    var arguments = argList.subList(1, argList.size()).toArray(new String[0]);

                    // Run compiler
                    var baos = new ByteArrayOutputStream();
                    var exitCode = compiler.run(null, baos, baos, arguments);

                    // Write compiler exit code and output back to client
                    channel.write(ByteBuffer.wrap((exitCode + "\n").getBytes(StandardCharsets.UTF_8)));
                    channel.write(ByteBuffer.wrap(baos.toByteArray()));
                    channel.close();
                } catch (Exception e) {
                    e.printStackTrace();
                    System.exit(1);
                }
            }).start();
        }
    }

    // Parse command line into arguments, handling quotes
    private static List<String> parseCommand(String cmd) {
        var result = new ArrayList<String>();
        var current = new StringBuilder();
        var inQuote = false;
        for (char c : cmd.toCharArray()) {
            if (c == '"') {
                inQuote = !inQuote;
            } else if (c == ' ' && !inQuote) {
                if (current.length() > 0) {
                    result.add(current.toString());
                    current.setLength(0);
                }
            } else {
                current.append(c);
            }
        }
        if (current.length() > 0) {
            result.add(current.toString());
        }
        return result;
    }
}
