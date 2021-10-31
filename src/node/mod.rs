pub use local_node::CanOpenNode;

mod local_node;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NodeId(u8);

impl NodeId {
    pub fn new(node_id: u8) -> Option<Self> {
        if node_id <= 127 {
            Some(NodeId(node_id))
        } else {
            None
        }
    }

    /// # Safety
    /// node_id <= 127
    pub const unsafe fn new_unchecked(node_id: u8) -> Self {
        NodeId(node_id)
    }

    pub const fn raw(&self) -> u8 {
        self.0
    }

    pub const NODE_ID_0: NodeId = NodeId(0);
    pub const NODE_ID_1: NodeId = NodeId(1);
    pub const NODE_ID_2: NodeId = NodeId(2);
    pub const NODE_ID_3: NodeId = NodeId(3);
    pub const NODE_ID_4: NodeId = NodeId(4);
    pub const NODE_ID_5: NodeId = NodeId(5);
    pub const NODE_ID_6: NodeId = NodeId(6);
    pub const NODE_ID_7: NodeId = NodeId(7);
    pub const NODE_ID_8: NodeId = NodeId(8);
    pub const NODE_ID_9: NodeId = NodeId(9);
    pub const NODE_ID_10: NodeId = NodeId(10);
    pub const NODE_ID_11: NodeId = NodeId(11);
    pub const NODE_ID_12: NodeId = NodeId(12);
    pub const NODE_ID_13: NodeId = NodeId(13);
    pub const NODE_ID_14: NodeId = NodeId(14);
    pub const NODE_ID_15: NodeId = NodeId(15);
    pub const NODE_ID_16: NodeId = NodeId(16);
    pub const NODE_ID_17: NodeId = NodeId(17);
    pub const NODE_ID_18: NodeId = NodeId(18);
    pub const NODE_ID_19: NodeId = NodeId(19);
    pub const NODE_ID_20: NodeId = NodeId(20);
    pub const NODE_ID_21: NodeId = NodeId(21);
    pub const NODE_ID_22: NodeId = NodeId(22);
    pub const NODE_ID_23: NodeId = NodeId(23);
    pub const NODE_ID_24: NodeId = NodeId(24);
    pub const NODE_ID_25: NodeId = NodeId(25);
    pub const NODE_ID_26: NodeId = NodeId(26);
    pub const NODE_ID_27: NodeId = NodeId(27);
    pub const NODE_ID_28: NodeId = NodeId(28);
    pub const NODE_ID_29: NodeId = NodeId(29);
    pub const NODE_ID_30: NodeId = NodeId(30);
    pub const NODE_ID_31: NodeId = NodeId(31);
    pub const NODE_ID_32: NodeId = NodeId(32);
    pub const NODE_ID_33: NodeId = NodeId(33);
    pub const NODE_ID_34: NodeId = NodeId(34);
    pub const NODE_ID_35: NodeId = NodeId(35);
    pub const NODE_ID_36: NodeId = NodeId(36);
    pub const NODE_ID_37: NodeId = NodeId(37);
    pub const NODE_ID_38: NodeId = NodeId(38);
    pub const NODE_ID_39: NodeId = NodeId(39);
    pub const NODE_ID_40: NodeId = NodeId(40);
    pub const NODE_ID_41: NodeId = NodeId(41);
    pub const NODE_ID_42: NodeId = NodeId(42);
    pub const NODE_ID_43: NodeId = NodeId(43);
    pub const NODE_ID_44: NodeId = NodeId(44);
    pub const NODE_ID_45: NodeId = NodeId(45);
    pub const NODE_ID_46: NodeId = NodeId(46);
    pub const NODE_ID_47: NodeId = NodeId(47);
    pub const NODE_ID_48: NodeId = NodeId(48);
    pub const NODE_ID_49: NodeId = NodeId(49);
    pub const NODE_ID_50: NodeId = NodeId(50);
    pub const NODE_ID_51: NodeId = NodeId(51);
    pub const NODE_ID_52: NodeId = NodeId(52);
    pub const NODE_ID_53: NodeId = NodeId(53);
    pub const NODE_ID_54: NodeId = NodeId(54);
    pub const NODE_ID_55: NodeId = NodeId(55);
    pub const NODE_ID_56: NodeId = NodeId(56);
    pub const NODE_ID_57: NodeId = NodeId(57);
    pub const NODE_ID_58: NodeId = NodeId(58);
    pub const NODE_ID_59: NodeId = NodeId(59);
    pub const NODE_ID_60: NodeId = NodeId(60);
    pub const NODE_ID_61: NodeId = NodeId(61);
    pub const NODE_ID_62: NodeId = NodeId(62);
    pub const NODE_ID_63: NodeId = NodeId(63);
    pub const NODE_ID_64: NodeId = NodeId(64);
    pub const NODE_ID_65: NodeId = NodeId(65);
    pub const NODE_ID_66: NodeId = NodeId(66);
    pub const NODE_ID_67: NodeId = NodeId(67);
    pub const NODE_ID_68: NodeId = NodeId(68);
    pub const NODE_ID_69: NodeId = NodeId(69);
    pub const NODE_ID_70: NodeId = NodeId(70);
    pub const NODE_ID_71: NodeId = NodeId(71);
    pub const NODE_ID_72: NodeId = NodeId(72);
    pub const NODE_ID_73: NodeId = NodeId(73);
    pub const NODE_ID_74: NodeId = NodeId(74);
    pub const NODE_ID_75: NodeId = NodeId(75);
    pub const NODE_ID_76: NodeId = NodeId(76);
    pub const NODE_ID_77: NodeId = NodeId(77);
    pub const NODE_ID_78: NodeId = NodeId(78);
    pub const NODE_ID_79: NodeId = NodeId(79);
    pub const NODE_ID_80: NodeId = NodeId(80);
    pub const NODE_ID_81: NodeId = NodeId(81);
    pub const NODE_ID_82: NodeId = NodeId(82);
    pub const NODE_ID_83: NodeId = NodeId(83);
    pub const NODE_ID_84: NodeId = NodeId(84);
    pub const NODE_ID_85: NodeId = NodeId(85);
    pub const NODE_ID_86: NodeId = NodeId(86);
    pub const NODE_ID_87: NodeId = NodeId(87);
    pub const NODE_ID_88: NodeId = NodeId(88);
    pub const NODE_ID_89: NodeId = NodeId(89);
    pub const NODE_ID_90: NodeId = NodeId(90);
    pub const NODE_ID_91: NodeId = NodeId(91);
    pub const NODE_ID_92: NodeId = NodeId(92);
    pub const NODE_ID_93: NodeId = NodeId(93);
    pub const NODE_ID_94: NodeId = NodeId(94);
    pub const NODE_ID_95: NodeId = NodeId(95);
    pub const NODE_ID_96: NodeId = NodeId(96);
    pub const NODE_ID_97: NodeId = NodeId(97);
    pub const NODE_ID_98: NodeId = NodeId(98);
    pub const NODE_ID_99: NodeId = NodeId(99);
    pub const NODE_ID_100: NodeId = NodeId(100);
    pub const NODE_ID_101: NodeId = NodeId(101);
    pub const NODE_ID_102: NodeId = NodeId(102);
    pub const NODE_ID_103: NodeId = NodeId(103);
    pub const NODE_ID_104: NodeId = NodeId(104);
    pub const NODE_ID_105: NodeId = NodeId(105);
    pub const NODE_ID_106: NodeId = NodeId(106);
    pub const NODE_ID_107: NodeId = NodeId(107);
    pub const NODE_ID_108: NodeId = NodeId(108);
    pub const NODE_ID_109: NodeId = NodeId(109);
    pub const NODE_ID_110: NodeId = NodeId(110);
    pub const NODE_ID_111: NodeId = NodeId(111);
    pub const NODE_ID_112: NodeId = NodeId(112);
    pub const NODE_ID_113: NodeId = NodeId(113);
    pub const NODE_ID_114: NodeId = NodeId(114);
    pub const NODE_ID_115: NodeId = NodeId(115);
    pub const NODE_ID_116: NodeId = NodeId(116);
    pub const NODE_ID_117: NodeId = NodeId(117);
    pub const NODE_ID_118: NodeId = NodeId(118);
    pub const NODE_ID_119: NodeId = NodeId(119);
    pub const NODE_ID_120: NodeId = NodeId(120);
    pub const NODE_ID_121: NodeId = NodeId(121);
    pub const NODE_ID_122: NodeId = NodeId(122);
    pub const NODE_ID_123: NodeId = NodeId(123);
    pub const NODE_ID_124: NodeId = NodeId(124);
    pub const NODE_ID_125: NodeId = NodeId(125);
    pub const NODE_ID_126: NodeId = NodeId(126);
    pub const NODE_ID_127: NodeId = NodeId(127);
}
