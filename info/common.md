### **基本功能 ( Common )**

基本功能查看機器人基本資訊和功能詳細指令。

* /info：查看機器人基本資訊
* /info [type]：查看功能詳細指令
  + Ex：/info reminder
* /ping：測試連線
* /update：查看更新日誌
  + all：查看所有更新日誌 (不可與 public 選項同時使用)
  + public：是否公開 (不可與 all 選項同時使用)
  + Ex：/update (查看最新日誌訊息)
  + Ex：/update all:true (查看所有更新日誌)
  + Ex：/update public:true (公開發送更新日誌)
* /ban：封禁成員
  + member_id：成員
  + time：封禁時間 ( 單位：秒 )
  + Ex：/ban member 10
* /unban：解封成員
  + member_id：成員
  + Ex：/unban member
