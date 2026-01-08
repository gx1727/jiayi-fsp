import React from 'react';
import { Link } from 'react-router-dom';
import { Github, Twitter } from 'lucide-react';

const Footer = () => {
  return (
    <footer className="bg-white border-t border-gray-100">
      <div className="container mx-auto px-4 py-12 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div className="col-span-1 md:col-span-2">
            <span className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-primary-600 to-secondary-600">
              JIAYI-FSP
            </span>
            <p className="mt-4 text-gray-500 text-sm max-w-md">
              专为家庭局域网设计的照片/视频同步协议。
              <br />
              可靠、隐私、速度。
            </p>
          </div>
          
          <div>
            <h3 className="text-sm font-semibold text-gray-400 tracking-wider uppercase">资源</h3>
            <ul className="mt-4 space-y-4">
              <li>
                <Link to="/specification" className="text-base text-gray-500 hover:text-gray-900">
                  技术规范
                </Link>
              </li>
              <li>
                <Link to="/developers" className="text-base text-gray-500 hover:text-gray-900">
                  开发者指南
                </Link>
              </li>
            </ul>
          </div>

          <div>
            <h3 className="text-sm font-semibold text-gray-400 tracking-wider uppercase">社区</h3>
            <ul className="mt-4 space-y-4">
              <li>
                <a href="#" className="text-base text-gray-500 hover:text-gray-900 flex items-center">
                  <Github className="w-4 h-4 mr-2" />
                  GitHub
                </a>
              </li>
            </ul>
          </div>
        </div>
        
        <div className="mt-8 border-t border-gray-100 pt-8 md:flex md:items-center md:justify-between">
          <p className="text-base text-gray-400">
            &copy; {new Date().getFullYear()} JIAYI-FSP. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
