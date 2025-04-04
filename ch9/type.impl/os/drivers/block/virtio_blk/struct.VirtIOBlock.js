(function() {
    var type_impls = Object.fromEntries([["os",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-BlockDevice-for-VirtIOBlock\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#17-67\">Source</a><a href=\"#impl-BlockDevice-for-VirtIOBlock\" class=\"anchor\">§</a><h3 class=\"code-header\">impl BlockDevice for <a class=\"struct\" href=\"os/drivers/block/virtio_blk/struct.VirtIOBlock.html\" title=\"struct os::drivers::block::virtio_blk::VirtIOBlock\">VirtIOBlock</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.read_block\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#18-38\">Source</a><a href=\"#method.read_block\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">read_block</a>(&amp;self, block_id: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>, buf: &amp;mut [<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u8.html\">u8</a>])</h4></section><section id=\"method.write_block\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#39-59\">Source</a><a href=\"#method.write_block\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">write_block</a>(&amp;self, block_id: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>, buf: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.u8.html\">u8</a>])</h4></section><section id=\"method.handle_irq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#60-66\">Source</a><a href=\"#method.handle_irq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">handle_irq</a>(&amp;self)</h4></section></div></details>","BlockDevice","os::board::BlockDeviceImpl"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-VirtIOBlock\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#69-87\">Source</a><a href=\"#impl-VirtIOBlock\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"os/drivers/block/virtio_blk/struct.VirtIOBlock.html\" title=\"struct os::drivers::block::virtio_blk::VirtIOBlock\">VirtIOBlock</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/os/drivers/block/virtio_blk.rs.html#70-86\">Source</a><h4 class=\"code-header\">pub fn <a href=\"os/drivers/block/virtio_blk/struct.VirtIOBlock.html#tymethod.new\" class=\"fn\">new</a>() -&gt; Self</h4></section></div></details>",0,"os::board::BlockDeviceImpl"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[2780]}