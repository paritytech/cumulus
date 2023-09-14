#!/bin/bash

# Name of the custom header file
custom_header="../custom_header.html"

# Find all .html files in current directory and its subdirectories
html_files=$(find . -type f -name "*.html")

# Iterate over each .html file
for file in $html_files
do
    # Check if the file contains <head> tag
    if grep -q "<head>" "$file" && grep -q "</head>" "$file"; then
        # Inject the contents of custom_header.html between <head> and </head> tags
        sed -i '/<head>/r '"$custom_header"'' "$file"
        echo "Custom header injected into $file"
    else
        echo "Skipped $file as it does not contain <head> tag"
    fi
done
