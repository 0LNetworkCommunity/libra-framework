prove:
    @cd move-stdlib && \
    echo "Testing move-stdlib" && \
    find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
    xargs -I {} sh -c 'echo "Testing file: {}"; diem move prove -f {} || echo "Error in file: {}"'


