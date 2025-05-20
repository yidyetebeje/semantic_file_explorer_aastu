import { Button } from "@/components/ui/button";
import { LucideIcon } from "lucide-react";

interface NavButtonProps {
  icon: LucideIcon;
  label: string;
  onClick?: () => void;
  active?: boolean;
}

const NavButton = ({ icon: Icon, label, onClick, active = false }: NavButtonProps) => (
  <Button 
    variant="ghost" 
    className={`w-full justify-start ${active ? 'bg-gray-800 text-white font-medium' : 'text-gray-300'} hover:text-gray-300 hover:bg-gray-800 cursor-pointer`}
    onClick={onClick}
  >
    <Icon className="mr-2 h-4 w-4" />
    {label}
  </Button>
);

export default NavButton;
