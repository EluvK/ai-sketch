### 观察了一下deepseek前端的stream API

Request URL:
https://chat.deepseek.com/api/v0/chat/edit_message
Request Method:
POST

```text
POST /api/v0/chat/edit_message HTTP/1.1
Accept-Encoding: gzip, deflate, br, zstd
Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7
Cache-Control: no-cache
Connection: keep-alive
Content-Length: 647

event: ready
data: {"stream_id":"f77ecf7f-bc83-4024-bf47-caa7056b6303"}

event: update_session
data: {"updated_at":1746693205.123147}

data: {"v": {"response": {"message_id": 17, "parent_id": 16, "model": "", "role": "ASSISTANT", "content": "", "thinking_enabled": true, "thinking_content": null, "thinking_elapsed_secs": null, "ban_edit": false, "ban_regenerate": false, "status": "WIP", "accumulated_token_usage": 0, "files": [], "tips": [], "inserted_at": 1746693205.107397, "search_enabled": false, "search_status": null, "search_results": null}}}

data: {"v": "好的", "p": "response/thinking_content"}

data: {"v": "，", "o": "APPEND"}

data: {"v": "用户"}

data: {"v": "希望"}

data: {"v": "我"}
// ...
data: {"v": 2672, "p": "response/accumulated_token_usage", "o": "SET"}

data: {"v": "FINISHED", "p": "response/status"}

event: finish
data: {}

event: update_session
data: {"updated_at":1746693272.343264}

event: close
data: {"click_behavior":"none"}
```


### stop api:

https://chat.deepseek.com/api/v0/chat/stop_stream
POST
{"chat_session_id":"7d2ca864-ca7a-4967-beb0-d9c86357224b","message_id":14}

resp: {"code":0,"msg":"","data":{"biz_code":0,"biz_msg":"","biz_data":null}}


### one chat sse

ready: with stream_id
update_session: updated_at timestamp

```text
event: ready
data: {"stream_id":"d8797609-a496-43ba-ae83-ce8fe3216cc8"}

event: update_session
data: {"updated_at":1746756083.999277}

data: {"v": {"response": {"message_id": 5, "parent_id": 4, "model": "", "role": "ASSISTANT", "content": "", "thinking_enabled": true, "thinking_content": null, "thinking_elapsed_secs": null, "ban_edit": false, "ban_regenerate": false, "status": "WIP", "accumulated_token_usage": 0, "files": [], "tips": [], "inserted_at": 1746756083.972602, "search_enabled": false, "search_status": null, "search_results": null}}}

data: {"v": "好的", "p": "response/thinking_content"}

data: {"v": "，", "o": "APPEND"}

data: {"v": "我现在"}

data: {"v": "需要"}

data: {"v": "解决的问题"}

data: {"v": "是"}

data: {"v": "来"}

data: {"v": "保证"}

data: {"v": "线程"}

data: {"v": "安全"}

data: {"v": "。"}

data: {"v": 50, "p": "response/thinking_elapsed_secs", "o": "SET"}

data: {"v": "以下是", "p": "response/content", "o": "APPEND"}

data: {"v": "分"}

data: {"v": "步"}

data: {"v": "解决方案"}

data: {"v": "数据"}

data: {"v": "。"}

data: {"v": 2297, "p": "response/accumulated_token_usage", "o": "SET"}

data: {"v": "FINISHED", "p": "response/status"}

event: finish
data: {}

event: update_session
data: {"updated_at":1746756192.784966}

event: close
data: {"click_behavior":"none"}

```

### resume api:

https://chat.deepseek.com/api/v0/chat/resume_stream
{"chat_session_id":"3d95ec3b-4a4b-4c89-9766-40c216902ca0","message_id":8}
Request Method:
POST
```json
{
    "v": {
        "response": {
            "message_id": 8,
            "parent_id": 7,
            "model": "",
            "role": "ASSISTANT",
            "content": "当客户端连接中断或刷新时，服务器端的行为如下：\n\n---\n\n### 1. **连接中断时的自动清理**\n- **接收者自动释放**：每个 SSE 连接会创建一个独立的广播通道接收者 (`rx`)。当客户端",
            "thinking_enabled": true,
            "thinking_content": "好的，我现在要解决用户的问题：如果在生成数据的过程中，连接中断或客户端刷新了，会发生什么？用户之前已经得到了一个使用Axum和Tokio广播通道实现的SSE流的示例代码，现在他想了解在这种情况下系统的行为。\n\n首先，我需要回忆SSE（Server-Sent Events）的工作原理。SSE允许服务器向客户端推送事件，基于HTTP长连接。当客户端断开连接时，服务器需要检测到并停止发送数据，否则可能会浪费资源。\n\n用户之前的代码使用了Tokio的广播通道（broadcast channel）。广播通道允许多个接收者，每个接收者会获取发送者之后的消息。当客户端连接时，会创建一个新的广播接收者（rx），然后循环接收消息并发送给客户端。\n\n现在的问题是，如果客户端中途断开连接（比如关闭页面或刷新），服务器端如何处理？会不会有资源泄漏？比如未释放的接收者，或者发送者是否继续发送数据到已关闭的连接？\n\n首先，考虑客户端断开连接的情况。当客户端断开时，Axum的SSE处理器应该会停止当前的流。在代码中，流是一个异步生成器，循环接收广播消息。如果客户端断开，流会被丢弃，异步任务结束。此时，广播通道的接收者（rx）会被丢弃，触发其析构函数。Tokio的广播通道接收者在被丢弃时，会通知发送者，减少订阅者的计数。因此，不会有残留的接收者，发送者也不会继续向不存在的接收者发送数据。\n\n不过，需要确认Tokio的广播通道是否在接收者被丢弃时自动取消订阅。根据Tokio文档，是的。每个接收者独立工作，当接收者被丢弃时，它不再接收消息，发送者不会保留无效的接收者。\n\n接下来，考虑发送者的情况。发送者（tx）是Arc包装的，被多个线程共享。数据生成线程在循环中发送消息，不管是否有接收者。当没有接收者时，发送消息会返回错误，但代码中使用了let _ = producer_tx.send(message);，忽略了错误。因此，即使没有客户端连接，数据生成线程仍然会继续运行，发送消息到通道，但这些消息会被丢弃，因为没有接收者。这可能会有一定的资源消耗，但不会导致错误或泄漏，因为发送到无接收者的通道不会阻塞，只是返回错误。\n\n当客户端重新连接时，会创建一个新的接收者，开始接收新的消息。旧的消息可能会因为广播通道的容量限制（示例中设置为16）而被丢弃，如果客户端断开时间较长，可能会丢失部分消息。但这是预期的行为，因为广播通道设计如此，处理背压通过丢弃旧消息。\n\n另外，在SSE处理器函数中，当客户端断开连接时，流会被终止，循环退出，rx被丢弃，接收者自动取消订阅。因此，每个客户端连接对应独立的接收者，断开连接时资源会被正确清理。\n\n需要检查的潜在问题：如果大量客户端频繁连接和断开，是否会导致发送者或接收者的资源问题？Tokio的广播通道设计为多生产者多消费者，每个接收者独立管理，因此应该没有问题。发送者不会因为接收者的断开而受影响，只是继续发送消息，无接收者时消息被丢弃。\n\n另外，用户可能关心如果客户端断开后，服务器是否继续生成数据。根据代码，数据生成线程是独立运行的，不管是否有客户端连接，它都会每秒生成消息。这可能适用于需要持续生成数据的情况，但如果生成数据成本高，可能需要根据是否有活跃的接收者来调整。不过用户的问题中没有提到这点，所以当前行为是合理的。\n\n总结：当客户端断开时，对应的接收者会被丢弃，发送者不会继续向其发送数据，不会有资源泄漏。重新连接时，客户端会获得新的接收者，并从当前消息开始接收，可能丢失断开期间的消息，取决于通道容量。",
            "thinking_elapsed_secs": 41,
            "ban_edit": false,
            "ban_regenerate": false,
            "status": "WIP",
            "accumulated_token_usage": 0,
            "files": [],
            "tips": [],
            "inserted_at": 1746757754.72831,
            "search_enabled": false,
            "search_status": null,
            "search_results": null
        }
    }
}
```
