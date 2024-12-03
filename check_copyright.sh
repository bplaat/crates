#!/bin/sh
EXIT=0
for file in $(find examples lib -name "*.rs"); do
    if ! grep -E -q "Copyright \(c\) 20[0-9]{2}(-20[0-9]{2})? Bastiaan van der Plaat" "$file"; then
        echo "Bad copyright header in: $file"
        EXIT=1
    fi
done
exit $EXIT
