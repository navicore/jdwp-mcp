#!/bin/bash
# Test script to see what JDWP sends when breakpoint hits

# 1. Connect and attach
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"debug.attach","arguments":{"host":"localhost","port":5005}}}'

# 2. Set breakpoint
sleep 0.5
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"debug.set_breakpoint","arguments":{"class_pattern":"com.example.probedemo.HelloController","line":64}}}'

# 3. List breakpoints
sleep 0.5
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"debug.list_breakpoints","arguments":{}}}'
