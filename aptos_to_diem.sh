#!/bin/bash

logfile="rename_changelog"

# Initialize log file
echo "Change log:" > $logfile

# Files and directories to process
files_and_dirs=$(find . -type f -not -path './.git*' -not -path './aptos_to_diem.sh' -not -path './rename_changelog' -o -type d -not -path './.git*')

# Process each file or directory
for fd in $files_and_dirs
do
    # Replace 'aptos' and 'Aptos' with 'diem' and 'Diem' respectively in files
    if [ -f "$fd" ]
    then
        # Replace in file
        sed -i -e 's/aptos/diem/g' -e 's/Aptos/Diem/g' -e 's/APTOS/DIEM/g' "$fd"

        # If a replacement was made
        if [ $? -eq 0 ]
        then
            echo "Made replacements in file: $fd" >> $logfile
        fi
    fi

    # Replace 'aptos' and 'Aptos' in directory names
    if [ -d "$fd" ]
    then
        new_fd=$(echo "$fd" | sed -e 's/aptos/diem/g' -e 's/Aptos/Diem/g' -e 's/APTOS/DIEM/g')
        
        # If the directory name was changed
        if [ "$fd" != "$new_fd" ]
        then
            # Rename directory
            mv "$fd" "$new_fd"

            # If the rename was successful
            if [ $? -eq 0 ]
            then
                echo "Renamed directory from: $fd to: $new_fd" >> $logfile
            fi
        fi
    fi
done

