eval "$(cat "$(dirname "${BASH_SOURCE[0]}")/../../.banksh/lib/exceptions")"

function alpha {
  beta -v2
}

function beta {
  try
    gamma -m "hello"
  catch TypeError
    echo "Oops, we gave the wrong type to gamma! ($(caught))"
    echo "Let's pretend that didn't happen."
  yrt
}

function gamma {
  delta --diffstat
}

function delta {
  throw TypeError: "got a cat when we expected a dog"
}

alpha
