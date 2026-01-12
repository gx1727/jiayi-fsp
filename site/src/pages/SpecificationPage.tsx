import React from 'react';
import { motion } from 'framer-motion';
import MessageDefinitions from '../components/specs/MessageDefinitions';
import UDPPacketStructure from '../components/specs/UDPPacketStructure';
import ReliabilityMechanisms from '../components/specs/ReliabilityMechanisms';
import ProtocolSequence from '../components/specs/ProtocolSequence';

const SpecificationPage = () => {
  return (
    <div className="bg-gray-50 min-h-screen py-16">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="text-center mb-12"
        >
          <h1 className="text-4xl font-extrabold text-gray-900 sm:text-5xl">
            技术规范
          </h1>
          <p className="mt-4 text-xl text-gray-500 max-w-2xl mx-auto">
            JIAYI-FSP v0.1 详细协议定义。
          </p>
        </motion.div>

        <div className="max-w-5xl mx-auto space-y-16">
          <section id="sequence">
             <ProtocolSequence />
          </section>

          <section id="messages" className="bg-white rounded-2xl shadow-sm p-8 border border-gray-100">
            <MessageDefinitions />
          </section>

          <section id="udp" className="bg-white rounded-2xl shadow-sm p-8 border border-gray-100">
            <UDPPacketStructure />
          </section>

          <section id="reliability">
            <ReliabilityMechanisms />
          </section>
        </div>
      </div>
    </div>
  );
};

export default SpecificationPage;
