const SidebarSection = ({ title, children }: { title: string; children: React.ReactNode }) => (
    <div>
      <h2 className="mb-2 px-2 text-lg font-semibold tracking-tight text-gray-300">
        {title}
      </h2>
      <div className="space-y-1">{children}</div>
    </div>
  );
  
  export default SidebarSection;
  