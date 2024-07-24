## ビルド方法
  事前準備は下記URLのREADME参照  
  https://github.com/hogehogei/milkv-duo-rust-helloworld

  ビルドコマンド
  ```
  $ RUST_BACKTRACE=1 cargo +nightly build --target riscv64gc-unknown-linux-musl -Zbuild-std --release
  ```

## Milk-V Duo 設定
- ポート設定
```
$ duo-pinmux -w GP8/IIC1_SDA
$ duo-pinmux -w GP9/IIC1_SCL
```
- 動作確認方法
  Linux ホストマシンに MilkV-Duo をつないで確認。
  - ネットワーク設定  
    Linux ホスト側
    MilkV-Duo をUSB接続し、"usb0" として見えていること    
    外側のインターネットは eth0 として見えていること    
    ```
    eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 192.168.24.107  netmask 255.255.255.0  broadcast 192.168.24.255
        inet6 2001:a451:9205:2c00:5c5a:dc25:d755:458f  prefixlen 64  scopeid 0x0<global>
        inet6 fe80::9871:9eea:5da2:341  prefixlen 64  scopeid 0x20<link>
        ether b8:27:eb:10:c9:88  txqueuelen 1000  (Ethernet)
        RX packets 947  bytes 604753 (590.5 KiB)
        RX errors 0  dropped 12  overruns 0  frame 0
        TX packets 560  bytes 65972 (64.4 KiB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0
    usb0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 192.168.42.218  netmask 255.255.255.0  broadcast 192.168.42.255
        inet6 fe80::721a:1e1:ea87:a4b0  prefixlen 64  scopeid 0x20<link>
        ether f2:b9:12:26:b2:37  txqueuelen 1000  (Ethernet)
        RX packets 110  bytes 12150 (11.8 KiB)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 183  bytes 26773 (26.1 KiB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0
    ```
    
    usb0 の IPアドレスを 192.168.42.2/24 に設定    
    usb0 をNATして、MilkV-Duo からの通信が外に出れるように設定  
    ```
    $ sudo ip addr add dev usb0 192.168.42.2/24
    $ sudo nft add table ip nat
    $ sudo nft add chain nat postrouting { type nat hook postrouting priority 100 \; }
    $ sudo nft add rule ip nat postrouting oif "usb0"  masquerade
    $ sudo nft add rule ip nat postrouting oif "eth0"  masquerade
    $ sudo nft add table inet filter
    $ sudo nft add chain inet filter forward { type filter hook forward priority 0\; policy accept \; }
    $ sudo nft add rule inet filter forward iif "usb0" ip saddr 192.168.42.1 oif "eth0" accept
    $ sudo nft add rule inet filter forward iif "eth0" oif "usb0" accept
    ```

    MilkV-Duo 側は以下のようにIPアドレス(192.168.42.1)、GW、DNSを設定  
    ```
    $ ifconfig
    usb0      Link encap:Ethernet  HWaddr 02:E6:84:D1:28:A3
          inet addr:192.168.42.1  Bcast:192.168.42.255  Mask:255.255.255.0
          inet6 addr: fe80::e6:84ff:fed1:28a3/64 Scope:Link
          UP BROADCAST RUNNING MULTICAST  MTU:1500  Metric:1
          RX packets:322 errors:0 dropped:0 overruns:0 frame:0
          TX packets:203 errors:0 dropped:0 overruns:0 carrier:0
          collisions:0 txqueuelen:1000
          RX bytes:27826 (27.1 KiB)  TX bytes:40650 (39.6 KiB)
    
    $ route add default gw 192.168.42.2
    $ echo "nameserver 8.8.8.8" >> /etc/resolv.conf
    ```
    
  - バイナリの実行  
    MilkV-Duo でバイナリを実行する場合、以下のように環境変数、TLS証明書を配置すること   
    ```
    $ export AWS_IOT_CLIENT_ID='your client id'
    $ export AWS_IOT_ENDPOINT='your AWS API endpoint URL'
    [root@milkv-duo]~# ls
    AmazonRootCA1.pem      milkv-duo-test.private.key
    milkv-duo-rust-bme280  milkv-duo-test.cert.pem
    ```
