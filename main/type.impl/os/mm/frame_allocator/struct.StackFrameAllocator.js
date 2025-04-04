(function() {
    var type_impls = Object.fromEntries([["os",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-FrameAllocator-for-StackFrameAllocator\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#55-92\">Source</a><a href=\"#impl-FrameAllocator-for-StackFrameAllocator\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"os/mm/frame_allocator/trait.FrameAllocator.html\" title=\"trait os::mm::frame_allocator::FrameAllocator\">FrameAllocator</a> for <a class=\"struct\" href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html\" title=\"struct os::mm::frame_allocator::StackFrameAllocator\">StackFrameAllocator</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#56-62\">Source</a><a href=\"#method.new\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.new\" class=\"fn\">new</a>() -&gt; Self</h4></section><section id=\"method.alloc\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#63-72\">Source</a><a href=\"#method.alloc\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.alloc\" class=\"fn\">alloc</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>&gt;</h4></section><section id=\"method.alloc_more\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#73-82\">Source</a><a href=\"#method.alloc_more\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.alloc_more\" class=\"fn\">alloc_more</a>(&amp;mut self, pages: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.usize.html\">usize</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>&gt;&gt;</h4></section><section id=\"method.dealloc\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#83-91\">Source</a><a href=\"#method.dealloc\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"os/mm/frame_allocator/trait.FrameAllocator.html#tymethod.dealloc\" class=\"fn\">dealloc</a>(&amp;mut self, ppn: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>)</h4></section></div></details>","FrameAllocator","os::mm::frame_allocator::FrameAllocatorImpl"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-StackFrameAllocator\" class=\"impl\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#48-54\">Source</a><a href=\"#impl-StackFrameAllocator\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html\" title=\"struct os::mm::frame_allocator::StackFrameAllocator\">StackFrameAllocator</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.init\" class=\"method\"><a class=\"src rightside\" href=\"src/os/mm/frame_allocator.rs.html#49-53\">Source</a><h4 class=\"code-header\">pub fn <a href=\"os/mm/frame_allocator/struct.StackFrameAllocator.html#tymethod.init\" class=\"fn\">init</a>(&amp;mut self, l: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>, r: <a class=\"struct\" href=\"os/mm/address/struct.PhysPageNum.html\" title=\"struct os::mm::address::PhysPageNum\">PhysPageNum</a>)</h4></section></div></details>",0,"os::mm::frame_allocator::FrameAllocatorImpl"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[4291]}