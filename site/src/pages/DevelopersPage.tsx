import React from 'react';
import { motion } from 'framer-motion';
import ImplementationGuide from '../components/developers/ImplementationGuide';
import ReferenceImplementation from '../components/developers/ReferenceImplementation';
import OpenSourceInfo from '../components/developers/OpenSourceInfo';

const DevelopersPage = () => {
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
            开发者资源
          </h1>
          <p className="mt-4 text-xl text-gray-500 max-w-2xl mx-auto">
            构建您自己的 JIAYI-FSP 实现。
          </p>
        </motion.div>

        <div className="max-w-4xl mx-auto">
          <ImplementationGuide />
          <ReferenceImplementation />
          <OpenSourceInfo />
        </div>
      </div>
    </div>
  );
};

export default DevelopersPage;
