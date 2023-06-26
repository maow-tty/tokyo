if [[ $1 == "bios" || $1 = "uefi" ]]
then
  cargo run --bin qemu-$1
fi
printf "Invalid or missing argument at 1st position, expected:\n  uefi = Launch QEMU with the UEFI image\n  bios = Launch QEMU with the BIOS image\n"
exit -1