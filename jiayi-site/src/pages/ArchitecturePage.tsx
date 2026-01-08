import React from 'react';
import ArchitectureDiagram from '../components/architecture/ArchitectureDiagram';
import StateMachine from '../components/architecture/StateMachine';
import DesignPrinciples from '../components/architecture/DesignPrinciples';
import { motion } from 'framer-motion';

const ArchitecturePage = () => {
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
            协议架构
          </h1>
          <p className="mt-4 text-xl text-gray-500 max-w-2xl mx-auto">
            深入了解 JIAYI-FSP 的内部构造，探索 MQTT 与 UDP 如何协同工作。
          </p>
        </motion.div>

        <div className="max-w-4xl mx-auto">
          <ArchitectureDiagram />
          <DesignPrinciples />
          <StateMachine />
        </div>
      </div>
    </div>
  );
};

export default ArchitecturePage;
