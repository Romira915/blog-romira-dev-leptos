watch:
    #!/bin/bash -eux
    stylance --watch . &
    cargo leptos watch &
    trap "kill %1; kill %2" EXIT
    wait
