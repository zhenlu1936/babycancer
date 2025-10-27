# sudo mknod tests/my_char_device c 1 7
# sudo chmod 777 tests/my_char_device

sudo mkfifo tests/my_fifo
sudo chmod 777 tests/my_fifo

ln -s tests/rplayer.png tests/my_symlink