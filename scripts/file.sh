sudo mknod testsrc/my_char_device c 1 7
sudo chmod 777 testsrc/my_char_device

sudo mkfifo testsrc/my_fifo
sudo chmod 777 testsrc/my_fifo

ln -s testsrc/rplayer.png testsrc/my_symlink