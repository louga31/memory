define dbg
  source ./load-symbols.py
  file
  load-symbols $rip "./target/x86_64-unknown-uefi/debug/memory.efi"
  set GDB_ATTACHED = 1
end