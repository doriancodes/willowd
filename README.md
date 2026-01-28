# Willowd

```shell
docker build -t willow-sys .

# --cap-add SYS_ADMIN : Allows the container to run 'mount'
# --device /dev/fuse  : Required if we ever switch to FUSE
docker run -it --rm --cap-add SYS_ADMIN --name willow-dev willow-sys /bin/bash

# 1. Start your Daemon in the background
# The logging will print to stdout, so we append '&'
./target/release/willowd &

# Wait a second for it to start...
# Output: [*] Listening on TCP 0.0.0.0:5640

# 2. Create the mount point
mkdir /mnt/willow

# 3. Mount it! (This command works because we are in Linux)
# trans=tcp : Use TCP transport
# port=5640 : Connect to localhost inside the container
mount -t 9p -o trans=tcp,port=5640,version=9p2000.L,uname=root,access=any 127.0.0.1 /mnt/willow

# 4. Verify it works
echo "Hello from Docker" > /mnt/willow/test.txt
cat /mnt/willow/test.txt
# Output: Hello from Docker

# 5. List the Virtual Filesystem
ls -la /mnt/willow
```
