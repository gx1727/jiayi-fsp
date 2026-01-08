import React from 'react';
import Hero from '../components/home/Hero';
import Features from '../components/home/Features';
import ArchitecturePreview from '../components/home/ArchitecturePreview';

const HomePage = () => {
  return (
    <div className="flex flex-col">
      <Hero />
      <Features />
      <ArchitecturePreview />
    </div>
  );
};

export default HomePage;
