
prove:
    @cd move-stdlib && \
    echo "Testing move-stdlib" && \
    find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
    xargs -I {} sh -c 'echo "Testing file: {}"; diem move prove -f {} || echo "Error in file: {}"'

    @cd vendor-stdlib && \
    echo "Testing vendor-stdlib" && \
    find sources -type f -name "*.move" ! -name "*.spec.move" | sed 's/\.move$$//' | \
    xargs -I {} sh -c 'echo "Testing file: {}"; diem move prove -f {} || echo "Error in file: {}"'

    @cd libra-framework && \
    echo "Testing libra-framework" && \
    find . -type f -name "*.move" ! -name "*.spec.move" -print0 | \
    xargs -0 -I {} sh -c 'echo "Testing file: {}"; diem move prove -f {} || echo "Error in file: {}"'

