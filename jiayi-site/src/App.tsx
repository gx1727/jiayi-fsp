import React, { Suspense, lazy } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Layout from './components/layout/Layout';

// Lazy load pages
const HomePage = lazy(() => import('./pages/HomePage'));
const ArchitecturePage = lazy(() => import('./pages/ArchitecturePage'));
const SpecificationPage = lazy(() => import('./pages/SpecificationPage'));
const ScenariosPage = lazy(() => import('./pages/ScenariosPage'));
const DevelopersPage = lazy(() => import('./pages/DevelopersPage'));

// Loading component
const Loading = () => (
  <div className="flex items-center justify-center min-h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
  </div>
);

function App() {
  return (
    <Router basename="/jiayi-fsp">
      <Layout>
        <Suspense fallback={<Loading />}>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/architecture" element={<ArchitecturePage />} />
            <Route path="/specification" element={<SpecificationPage />} />
            <Route path="/scenarios" element={<ScenariosPage />} />
            <Route path="/developers" element={<DevelopersPage />} />
          </Routes>
        </Suspense>
      </Layout>
    </Router>
  );
}

export default App;
