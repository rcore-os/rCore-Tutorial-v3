#![allow(unused)]
#![allow(non_camel_case_types)]

//! DMAC peripheral
//use k210_hal::pac;
use k210_pac as pac;
use pac::dmac::channel::cfg::{TT_FC_A,HS_SEL_SRC_A};
use pac::dmac::channel::ctl::{SMS_A};

use super::sysctl;

/** Extension trait for adding configure() to DMAC peripheral */
pub trait DMACExt: Sized {
    /// Constrains DVP peripheral
    fn configure(self) -> DMAC;
}

impl DMACExt for pac::DMAC {
    fn configure(self) -> DMAC { DMAC::new(self) }
}

/** DMAC peripheral abstraction */
pub struct DMAC {
    dmac: pac::DMAC,
}

/*
typedef struct _dmac_context
{
    dmac_channel_number_t dmac_channel;
    plic_irq_callback_t callback;
    void *ctx;
} dmac_context_t;

dmac_context_t dmac_context[6];
*/

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum src_dst_select {
    SRC = 0x1,
    DST = 0x2,
    SRC_DST = 0x3,
}

pub use super::sysctl::dma_channel;
pub type master_number = pac::dmac::channel::ctl::SMS_A;
pub type address_increment = pac::dmac::channel::ctl::SINC_A;
pub type burst_length = pac::dmac::channel::ctl::SRC_MSIZE_A;
pub type transfer_width = pac::dmac::channel::ctl::SRC_TR_WIDTH_A;

/** Return whether a specific address considered considered memory or peripheral */
fn is_memory(address: u64) -> bool {
    let mem_len = 6 * 1024 * 1024;
    let mem_no_cache_len = 8 * 1024 * 1024;
    // Note: This comes from the Kendryte SDK as-is. No, I have no idea why the AES accelerator
    // input address 0x50450040 is considered memory, either.
    ((address >= 0x80000000) && (address < 0x80000000 + mem_len))
        || ((address >= 0x40000000) && (address < 0x40000000 + mem_no_cache_len))
        || (address == 0x50450040)
}

impl DMAC {
    fn new(dmac: pac::DMAC) -> Self {
        let rv = Self { dmac };
        rv.init();
        rv
    }

    /** Get DMAC ID */
    pub fn read_id(&self) -> u64 {
        return self.dmac.id.read().bits();
    }

    /** Get DMAC version */
    pub fn read_version(&self) -> u64 {
        return self.dmac.compver.read().bits();
    }

    /** Get AXI ID for channel */
    pub fn read_channel_id(&self, channel_num: dma_channel) -> u64 {
        return self.dmac.channel[channel_num.idx()].axi_id.read().bits();
    }

    /** Enable DMAC peripheral. */
    fn enable(&self) {
        self.dmac.cfg.modify(|_,w| w.dmac_en().set_bit()
            .int_en().set_bit());
    }

    /** Disable DMAC peripheral. */
    pub fn disable(&self) {
        self.dmac.cfg.modify(|_,w| w.dmac_en().clear_bit()
            .int_en().clear_bit());
    }

    pub fn src_transaction_complete_int_enable(&self, channel_num: dma_channel) {
        self.dmac.channel[channel_num.idx()].intstatus_en.modify(
            |_,w| w.src_transcomp().set_bit());
    }

    /** Enable a DMA channel. */
    pub fn channel_enable(&self, channel_num: dma_channel) {
        use dma_channel::*;
        // Note: chX bit names start counting from 1, while channels start counting from 0
        match channel_num {
            CHANNEL0 => {
                self.dmac.chen.modify(|_,w| w.ch1_en().set_bit()
                    .ch1_en_we().set_bit());
            }
            CHANNEL1 => {
                self.dmac.chen.modify(|_,w| w.ch2_en().set_bit()
                    .ch2_en_we().set_bit());
            }
            CHANNEL2 => {
                self.dmac.chen.modify(|_,w| w.ch3_en().set_bit()
                    .ch3_en_we().set_bit());
            }
            CHANNEL3 => {
                self.dmac.chen.modify(|_,w| w.ch4_en().set_bit()
                    .ch4_en_we().set_bit());
            }
            CHANNEL4 => {
                self.dmac.chen.modify(|_,w| w.ch5_en().set_bit()
                    .ch5_en_we().set_bit());
            }
            CHANNEL5 => {
                self.dmac.chen.modify(|_,w| w.ch6_en().set_bit()
                    .ch6_en_we().set_bit());
            }
        }
    }

    /** Disable a DMA channel. */
    pub fn channel_disable(&self, channel_num: dma_channel) {
        use dma_channel::*;
        // Note: chX bit names start counting from 1, while channels start counting from 0
        match channel_num {
            CHANNEL0 => {
                self.dmac.chen.modify(|_,w| w.ch1_en().clear_bit()
                    .ch1_en_we().set_bit());
            }
            CHANNEL1 => {
                self.dmac.chen.modify(|_,w| w.ch2_en().clear_bit()
                    .ch2_en_we().set_bit());
            }
            CHANNEL2 => {
                self.dmac.chen.modify(|_,w| w.ch3_en().clear_bit()
                    .ch3_en_we().set_bit());
            }
            CHANNEL3 => {
                self.dmac.chen.modify(|_,w| w.ch4_en().clear_bit()
                    .ch4_en_we().set_bit());
            }
            CHANNEL4 => {
                self.dmac.chen.modify(|_,w| w.ch5_en().clear_bit()
                    .ch5_en_we().set_bit());
            }
            CHANNEL5 => {
                self.dmac.chen.modify(|_,w| w.ch6_en().clear_bit()
                    .ch6_en_we().set_bit());
            }
        }
    }

    /** Check if a DMA channel is busy. */
    pub fn check_channel_busy(&self, channel_num: dma_channel) -> bool {
        use dma_channel::*;
        match channel_num {
            CHANNEL0 => self.dmac.chen.read().ch1_en().bit(),
            CHANNEL1 => self.dmac.chen.read().ch2_en().bit(),
            CHANNEL2 => self.dmac.chen.read().ch3_en().bit(),
            CHANNEL3 => self.dmac.chen.read().ch4_en().bit(),
            CHANNEL4 => self.dmac.chen.read().ch5_en().bit(),
            CHANNEL5 => self.dmac.chen.read().ch6_en().bit(),
        }
        // Note: Kendryte SDK writes back the value after reading it,
        // is this necessary? It seems not.
    }

    pub fn set_list_master_select(&self, channel_num: dma_channel, sd_sel: src_dst_select, mst_num: master_number) -> Result<(),()> {
        if !self.check_channel_busy(channel_num) {
            use src_dst_select::*;
            self.dmac.channel[channel_num.idx()].ctl.modify(|_,w| {
                let w = if sd_sel == SRC || sd_sel == SRC_DST {
                    w.sms().variant(mst_num)
                } else {
                    w
                };
                if sd_sel == DST || sd_sel == SRC_DST {
                    w.dms().variant(mst_num)
                } else {
                    w
                }
            });
            // Note: there's some weird tmp|= line here in the Kendryte SDK
            // with the result going unused. I've decided to leave this out
            // because I assume it's another C UB workaround.
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn enable_common_interrupt_status(&self) {
        self.dmac.com_intstatus_en.modify(|_,w|
            w.slvif_dec_err().set_bit()
                .slvif_wr2ro_err().set_bit()
                .slvif_rd2wo_err().set_bit()
                .slvif_wronhold_err().set_bit()
                .slvif_undefinedreg_dec_err().set_bit()
        );
    }

    pub fn enable_common_interrupt_signal(&self) {
        self.dmac.com_intsignal_en.modify(|_,w|
            w.slvif_dec_err().set_bit()
                .slvif_wr2ro_err().set_bit()
                .slvif_rd2wo_err().set_bit()
                .slvif_wronhold_err().set_bit()
                .slvif_undefinedreg_dec_err().set_bit()
        );
    }

    fn enable_channel_interrupt(&self, channel_num: dma_channel) {
        unsafe {
            let ch = &self.dmac.channel[channel_num.idx()];
            ch.intclear.write(|w| w.bits(0xffffffff));
            ch.intstatus_en.write(|w| w.bits(0x2));
        }
    }

    pub fn disable_channel_interrupt(&self, channel_num: dma_channel) {
        unsafe {
            self.dmac.channel[channel_num.idx()].intstatus_en.write(
                |w| w.bits(0x0));
        }
    }

    fn channel_interrupt_clear(&self, channel_num: dma_channel) {
        unsafe {
            self.dmac.channel[channel_num.idx()].intclear.write(
                |w| w.bits(0xffffffff));
        }
    }

    /** Set DMA channel parameters. */
    pub fn set_channel_param(&self, channel_num: dma_channel,
                             src: u64, dest: u64, src_inc: address_increment, dest_inc: address_increment,
                             burst_size: burst_length,
                             trans_width: transfer_width,
                             block_size: u32) {
        unsafe {
            let ch = &self.dmac.channel[channel_num.idx()];
            let src_is_mem = is_memory(src);
            let dest_is_mem = is_memory(dest);
            let flow_control = match (src_is_mem, dest_is_mem) {
                (false, false) => TT_FC_A::PRF2PRF_DMA,
                (true, false) => TT_FC_A::MEM2PRF_DMA,
                (false, true) => TT_FC_A::PRF2MEM_DMA,
                (true, true) => TT_FC_A::MEM2MEM_DMA,
            };

            /*
             * cfg register must configure before ts_block and
             * sar dar register
             */
            ch.cfg.modify(|_,w|
                w.tt_fc().variant(flow_control)
                    .hs_sel_src().variant(if src_is_mem { HS_SEL_SRC_A::SOFTWARE } else { HS_SEL_SRC_A::HARDWARE } )
                    .hs_sel_dst().variant(if dest_is_mem { HS_SEL_SRC_A::SOFTWARE } else { HS_SEL_SRC_A::HARDWARE } )
                    // Note: from SVD: "Assign a hardware handshaking interface to source of channel",
                    // these are set using sysctl::dma_select; this configuration seems to indicate
                    // that in principle, it's possible to use a different source and destination
                    // handshaking interface for a channel, but that would sacrifice the interface of
                    // another channel.
                    .src_per().bits(channel_num as u8)
                    .dst_per().bits(channel_num as u8)
                    .src_multblk_type().bits(0)
                    .dst_multblk_type().bits(0)
            );

            ch.sar.write(|w| w.bits(src));
            ch.dar.write(|w| w.bits(dest));

            ch.ctl.modify(|_,w|
                w.sms().variant(SMS_A::AXI_MASTER_1)
                    .dms().variant(SMS_A::AXI_MASTER_2)
                    /* master select */
                    .sinc().variant(src_inc)
                    .dinc().variant(dest_inc)
                    /* address incrememt */
                    .src_tr_width().variant(trans_width)
                    .dst_tr_width().variant(trans_width)
                    /* transfer width */
                    .src_msize().variant(burst_size)
                    .dst_msize().variant(burst_size)
            );

            ch.block_ts.write(|w| w.block_ts().bits(block_size - 1));
            /*the number of (blcok_ts +1) data of width SRC_TR_WIDTF to be */
            /* transferred in a dma block transfer */
        }
    }

    /*
    pub void set_address(&self, dmac_channel_number_t channel_num, uint64_t src_addr,
                          uint64_t dst_addr) {
        writeq(src_addr, &dmac->channel[channel_num].sar);
        writeq(dst_addr, &dmac->channel[channel_num].dar);
    }
    */

    /*
    pub void set_block_ts(&self, dmac_channel_number_t channel_num,
                           uint32_t block_size) {
        uint32_t block_ts;

        block_ts = block_size & 0x3fffff;
        writeq(block_ts, &dmac->channel[channel_num].block_ts);
    }
    */

    /*
    pub void source_control(&self, dmac_channel_number_t channel_num,
                             dmac_master_number_t master_select,
                             dmac_address_increment_t address_mode,
                             dmac_transfer_width_t tr_width,
                             dmac_burst_length_t burst_length) {
        dmac_ch_ctl_u_t ctl_u;

        ctl_u.data = readq(&dmac->channel[channel_num].ctl);
        ctl_u.ch_ctl.sms = master_select;
        ctl_u.ch_ctl.sinc = address_mode;
        ctl_u.ch_ctl.src_tr_width = tr_width;
        ctl_u.ch_ctl.src_msize = burst_length;

        writeq(ctl_u.data, &dmac->channel[channel_num].ctl);
    }
    */

    /*
    pub void master_control(&self, dmac_channel_number_t channel_num,
                             dmac_master_number_t master_select,
                             dmac_address_increment_t address_mode,
                             dmac_transfer_width_t tr_width,
                             dmac_burst_length_t burst_length) {
        dmac_ch_ctl_u_t ctl_u;

        ctl_u.data = readq(&dmac->channel[channel_num].ctl);
        ctl_u.ch_ctl.dms = master_select;
        ctl_u.ch_ctl.dinc = address_mode;
        ctl_u.ch_ctl.dst_tr_width = tr_width;
        ctl_u.ch_ctl.dst_msize = burst_length;

        writeq(ctl_u.data, &dmac->channel[channel_num].ctl);
    }
    */

    /*
    pub void set_source_transfer_control(&self, dmac_channel_number_t channel_num,
                                          dmac_multiblk_transfer_type_t transfer_type,
                                          dmac_sw_hw_hs_select_t handshak_select) {
        dmac_ch_cfg_u_t cfg_u;

        cfg_u.data = readq(&dmac->channel[channel_num].cfg);
        cfg_u.ch_cfg.src_multblk_type = transfer_type;
        cfg_u.ch_cfg.hs_sel_src = handshak_select;

        writeq(cfg_u.data, &dmac->channel[channel_num].cfg);
    }
    */

    /*
    pub void set_destination_transfer_control(&self, dmac_channel_number_t channel_num,
                                               dmac_multiblk_transfer_type_t transfer_type,
                                               dmac_sw_hw_hs_select_t handshak_select) {
        dmac_ch_cfg_u_t cfg_u;

        cfg_u.data = readq(&dmac->channel[channel_num].cfg);
        cfg_u.ch_cfg.dst_multblk_type = transfer_type;
        cfg_u.ch_cfg.hs_sel_dst = handshak_select;

        writeq(cfg_u.data, &dmac->channel[channel_num].cfg);
    }
    */

    /*
    pub void set_flow_control(&self, dmac_channel_number_t channel_num,
                               dmac_transfer_flow_t flow_control) {
        dmac_ch_cfg_u_t cfg_u;

        cfg_u.data = readq(&dmac->channel[channel_num].cfg);
        cfg_u.ch_cfg.tt_fc = flow_control;

        writeq(cfg_u.data, &dmac->channel[channel_num].cfg);
    }
    */

    /*
    pub void set_linked_list_addr_point(&self, dmac_channel_number_t channel_num,
                                         uint64_t *addr) {
        dmac_ch_llp_u_t llp_u;

        llp_u.data = readq(&dmac->channel[channel_num].llp);
        /* Cast pointer to uint64_t */
        llp_u.llp.loc = (uint64_t)addr;
        writeq(llp_u.data, &dmac->channel[channel_num].llp);
    }
    */

    /** Initialize DMA controller */
    pub fn init(&self) {
        sysctl::clock_enable(sysctl::clock::DMA);

        /* reset dmac */
        self.dmac.reset.modify(|_,w| w.rst().set_bit());
        while self.dmac.reset.read().rst().bit() {
            // IDLE
        }

        /* clear common register interrupt */
        self.dmac.com_intclear.modify(|_,w|
            w.slvif_dec_err().set_bit()
                .slvif_wr2ro_err().set_bit()
                .slvif_rd2wo_err().set_bit()
                .slvif_wronhold_err().set_bit()
                .slvif_undefinedreg_dec_err().set_bit()
        );

        /* disable dmac and disable interrupt */
        self.dmac.cfg.modify(|_,w|
            w.dmac_en().clear_bit()
                .int_en().clear_bit()
        );

        while self.dmac.cfg.read().bits() != 0 {
            // IDLE
        }
        /* disable all channel before configure */
        /* Note: changed from the SDK code, which doesn't clear channel 4 and 5,
         * and doesn't set associated _we bits */
        self.dmac.chen.modify(|_,w|
            w.ch1_en().clear_bit()
                .ch1_en_we().set_bit()
                .ch2_en().clear_bit()
                .ch2_en_we().set_bit()
                .ch3_en().clear_bit()
                .ch3_en_we().set_bit()
                .ch4_en().clear_bit()
                .ch4_en_we().set_bit()
                .ch5_en().clear_bit()
                .ch5_en_we().set_bit()
        );

        self.enable();
    }

// TODO: list (scatter/gather) functionality

    /*
    static void list_add(struct list_head_t *new, struct list_head_t *prev,
                         struct list_head_t *next) {
        next->prev = new;
        new->next = next;
        new->prev = prev;
        prev->next = new;
    }
    */

    /*
    pub void list_add_tail(struct list_head_t *new, struct list_head_t *head) {
        list_add(new, head->prev, head);
    }
    */

    /*
    pub void INIT_LIST_HEAD(struct list_head_t *list) {
        list->next = list;
        list->prev = list;
    }
    */

    /*
    pub void link_list_item(dmac_channel_number_t channel_num,
                             uint8_t LLI_row_num, int8_t LLI_last_row,
                             dmac_lli_item_t *lli_item,
                             dmac_channel_config_t *cfg_param) {
        dmac_ch_ctl_u_t ctl;
        dmac_ch_llp_u_t llp_u;

        lli_item[LLI_row_num].sar = cfg_param->sar;
        lli_item[LLI_row_num].dar = cfg_param->dar;

        ctl.data = readq(&dmac->channel[channel_num].ctl);
        ctl.ch_ctl.sms = cfg_param->ctl_sms;
        ctl.ch_ctl.dms = cfg_param->ctl_dms;
        ctl.ch_ctl.sinc = cfg_param->ctl_sinc;
        ctl.ch_ctl.dinc = cfg_param->ctl_dinc;
        ctl.ch_ctl.src_tr_width = cfg_param->ctl_src_tr_width;
        ctl.ch_ctl.dst_tr_width = cfg_param->ctl_dst_tr_width;
        ctl.ch_ctl.src_msize = cfg_param->ctl_src_msize;
        ctl.ch_ctl.dst_msize = cfg_param->ctl_drc_msize;
        ctl.ch_ctl.src_stat_en = cfg_param->ctl_src_stat_en;
        ctl.ch_ctl.dst_stat_en = cfg_param->ctl_dst_stat_en;

        if(LLI_last_row != LAST_ROW)
        {
            ctl.ch_ctl.shadowreg_or_lli_valid = 1;
            ctl.ch_ctl.shadowreg_or_lli_last = 0;
        } else
        {
            ctl.ch_ctl.shadowreg_or_lli_valid = 1;
            ctl.ch_ctl.shadowreg_or_lli_last = 1;
        }

        lli_item[LLI_row_num].ctl = ctl.data;

        lli_item[LLI_row_num].ch_block_ts = cfg_param->ctl_block_ts;
        lli_item[LLI_row_num].sstat = 0;
        lli_item[LLI_row_num].dstat = 0;

        llp_u.data = readq(&dmac->channel[channel_num].llp);

        if(LLI_last_row != LAST_ROW)
            llp_u.llp.loc = ((uint64_t)&lli_item[LLI_row_num + 1]) >> 6;
        else
            llp_u.llp.loc = 0;

        lli_item[LLI_row_num].llp = llp_u.data;
    }

    pub void update_shadow_register(&self, dmac_channel_number_t channel_num,
                                      int8_t last_block, dmac_channel_config_t *cfg_param) {
        dmac_ch_ctl_u_t ctl_u;

        do
        {
            ctl_u.data = readq(&dmac->channel[channel_num].ctl);
        } while(ctl_u.ch_ctl.shadowreg_or_lli_valid);

        writeq(cfg_param->sar, &dmac->channel[channel_num].sar);
        writeq(cfg_param->dar, &dmac->channel[channel_num].dar);
        writeq(cfg_param->ctl_block_ts, &dmac->channel[channel_num].block_ts);

        ctl_u.ch_ctl.sms = cfg_param->ctl_sms;
        ctl_u.ch_ctl.dms = cfg_param->ctl_dms;
        ctl_u.ch_ctl.sinc = cfg_param->ctl_sinc;
        ctl_u.ch_ctl.dinc = cfg_param->ctl_dinc;
        ctl_u.ch_ctl.src_tr_width = cfg_param->ctl_src_tr_width;
        ctl_u.ch_ctl.dst_tr_width = cfg_param->ctl_dst_tr_width;
        ctl_u.ch_ctl.src_msize = cfg_param->ctl_src_msize;
        ctl_u.ch_ctl.dst_msize = cfg_param->ctl_drc_msize;
        ctl_u.ch_ctl.src_stat_en = cfg_param->ctl_src_stat_en;
        ctl_u.ch_ctl.dst_stat_en = cfg_param->ctl_dst_stat_en;
        if(last_block != LAST_ROW)
        {
            ctl_u.ch_ctl.shadowreg_or_lli_valid = 1;
            ctl_u.ch_ctl.shadowreg_or_lli_last = 0;
        } else
        {
            ctl_u.ch_ctl.shadowreg_or_lli_valid = 1;
            ctl_u.ch_ctl.shadowreg_or_lli_last = 1;
        }

        writeq(ctl_u.data, &dmac->channel[channel_num].ctl);
        writeq(0, &dmac->channel[channel_num].blk_tfr);
    }
    */

    /*
    pub void set_shadow_invalid_flag(&self, dmac_channel_number_t channel_num) {
        dmac_ch_ctl_u_t ctl_u;

        ctl_u.data = readq(&dmac->channel[channel_num].ctl);
        ctl_u.ch_ctl.shadowreg_or_lli_valid = 1;
        ctl_u.ch_ctl.shadowreg_or_lli_last = 0;
        writeq(ctl_u.data, &dmac->channel[channel_num].ctl);
    }
    */

    /** Start a single DMA transfer. */
    pub fn set_single_mode(&self, channel_num: dma_channel,
                           src: u64, dest: u64, src_inc: address_increment,
                           dest_inc: address_increment,
                           burst_size: burst_length,
                           trans_width: transfer_width,
                           block_size: u32) {
        self.channel_interrupt_clear(channel_num);
        self.channel_disable(channel_num);
        self.wait_idle(channel_num);
        self.set_channel_param(channel_num, src, dest, src_inc, dest_inc,
                               burst_size, trans_width, block_size);
        self.enable();
        self.channel_enable(channel_num);
    }

    /*
    pub int is_done(&self, dmac_channel_number_t channel_num) {
        if(readq(&dmac->channel[channel_num].intstatus) & 0x2)
            return 1;
        else
            return 0;
    }
    */

    /** Wait for dmac work done. */
    pub fn wait_done(&self, channel_num: dma_channel) {
        self.wait_idle(channel_num);
    }

    /** Determine if a DMA channel is idle or not. */
    pub fn is_idle(&self, channel_num: dma_channel) -> bool {
        !self.check_channel_busy(channel_num)
    }

    /** Wait for a DMA channel to be idle. */
    pub fn wait_idle(&self, channel_num: dma_channel) {
        while !self.is_idle(channel_num) {
        }
        self.channel_interrupt_clear(channel_num); /* clear interrupt */
    }

    /*
    pub void set_src_dest_length(&self, dmac_channel_number_t channel_num, const void *src, void *dest, size_t len) {
        if(src != NULL)
            dmac->channel[channel_num].sar = (uint64_t)src;
        if(dest != NULL)
            dmac->channel[channel_num].dar = (uint64_t)dest;
        if(len > 0)
            dmac_set_block_ts(channel_num, len - 1);
        dmac_channel_enable(channel_num);
    }
    */

// TODO: completion IRQ functionality

}
