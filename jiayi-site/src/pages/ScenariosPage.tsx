import React from 'react';
import { motion } from 'framer-motion';
import NetworkEnvironment from '../components/scenarios/NetworkEnvironment';
import DeviceRoles from '../components/scenarios/DeviceRoles';
import UseCases from '../components/scenarios/UseCases';

const ScenariosPage = () => {
  return (
    <div className="bg-white min-h-screen py-16">
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="text-center mb-12"
        >
          <h1 className="text-4xl font-extrabold text-gray-900 sm:text-5xl">
            应用场景
          </h1>
          <p className="mt-4 text-xl text-gray-500 max-w-2xl mx-auto">
            明确的边界与角色定义，让同步更简单。
          </p>
        </motion.div>

        <div className="max-w-4xl mx-auto">
          <NetworkEnvironment />
          <DeviceRoles />
          <UseCases />
        </div>
      </div>
    </div>
  );
};

export default ScenariosPage;
