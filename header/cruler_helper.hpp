#pragma once

#ifndef CRULER_HELPER_H
#define CRULER_HELPER_H

#include <iostream>
#include <vector>
#include "cruler.h"

enum class ProcedureName {
	NAVER_WEBTOON,
	DEFAULT,
};

class ExtractRuleBuilder {
public:
	ExtractRuleBuilder() : name(nullptr), 
		links(nullptr), local_path(nullptr), parts(nullptr), 
		extract(nullptr), post_procedure(ProcedureName::DEFAULT) {
		
	}
	virtual ~ExtractRuleBuilder() {
		if (name != nullptr) {
			delete name;
			name = nullptr;
		}

		if (links != nullptr) {
			delete links;
			links = nullptr;
		}

		if (local_path != nullptr) {
			delete local_path;
			local_path = nullptr;
		}

		if (parts != nullptr) {
			delete parts;
			parts = nullptr;
		}

		if (extract != nullptr) {
			delete extract;
			extract = nullptr;
		}
	}

	ExtractRuleBuilder* set_name(std::string& name) {
		if (this->name != nullptr) {
			delete this->name;
		}
		this->name = new std::string(name.c_str());
		return this;
	}

	ExtractRuleBuilder* add_link(std::string& link) {
		if (this->links == nullptr) {
			this->links = new std::vector<std::string>();
		}
		this->links->push_back(link);
		return this;
	}

	ExtractRuleBuilder* set_local_path(std::string& local_path) {
		if (this->local_path != nullptr) {
			delete this->local_path;
		}
		this->local_path = new std::string(local_path.c_str());
		return this;
	}
	
	ExtractRuleBuilder* add_part(std::string& link) {
		if (this->parts == nullptr) {
			this->parts = new std::vector<std::string>();
		}
		this->parts->push_back(link);
		return this;
	}

	ExtractRuleBuilder* set_extract(std::string& extract) {
		if (this->extract != nullptr) {
			delete this->extract;
		}
		this->extract = new std::string(extract.c_str());
		return this;
	}

	ExtractRuleBuilder* set_post_procedure(ProcedureName post_procedure) {
		this->post_procedure = post_procedure;
		return this;
	}

	std::string build() {
		if (!this->check_mandatory_params()) {
			return std::string("");
		}

		std::string rule_raw_string = "[[extraction]]\n";
		rule_raw_string += "name = '$<name>'\n";
		if (this->links != nullptr) {
			rule_raw_string += "links = [$<link>]\n";
		}
		rule_raw_string += "local_path = '$<local_path>'\n";
		if (this->parts != nullptr) {
			rule_raw_string += "parts = [$<parts>]\n";
		}
		rule_raw_string += "extract = '$<extract>'\n";
		if (this->post_procedure != ProcedureName::DEFAULT) {
			rule_raw_string += "[extraction.procedure]\n";
			rule_raw_string += "post_procedure = '$<post_procedure>'\n";
		}

		ExtractRuleBuilder::replace(rule_raw_string, "$<name>", *name);
		if (this->links != nullptr) {
			ExtractRuleBuilder::replace(rule_raw_string, "$<link>", ExtractRuleBuilder::get_array_replace_string(this->links));
		}
		ExtractRuleBuilder::replace(rule_raw_string, "$<local_path>", *this->local_path);
		if (this->parts != nullptr) {
			ExtractRuleBuilder::replace(rule_raw_string, "$<parts>", ExtractRuleBuilder::get_array_replace_string(this->parts));
		}
		ExtractRuleBuilder::replace(rule_raw_string, "$<extract>", *this->extract);
		if (this->post_procedure != ProcedureName::DEFAULT) {
			
		}
		switch (this->post_procedure) {
		case ProcedureName::NAVER_WEBTOON: {
			ExtractRuleBuilder::replace(rule_raw_string, "$<post_procedure>", "naver-webtoon");
		} break;
		default: { } break;
		}

		return rule_raw_string;
	}

protected:
	bool check_mandatory_params() {
		if (this->name == nullptr || this->local_path == nullptr || this->extract == nullptr) {
			return false;
		}
		return true;
	}

	bool replace(std::string& str, const std::string& from, const std::string& to) {
		size_t start_pos = str.find(from);
		if (start_pos == std::string::npos)
			return false;
		str.replace(start_pos, from.length(), to);
		return true;
	}

	std::string get_array_replace_string(std::vector<std::string>* vec) {
		size_t vector_size = vec->size();
		std::string output;
		for (std::vector<std::string>::iterator it = vec->begin(); it != vec->end(); ++it) {
			output += "\"" + *it + "\"";
			vector_size--;
			if (vector_size != 0) {
				output += ",\n";
			}
		}
		return output;
	}

private:
	std::string* name;
	std::vector<std::string>* links;
	std::string* local_path;
	std::vector<std::string>* parts;
	std::string* extract;
	ProcedureName post_procedure;
};

class CrulerHelper {
public:
	CrulerHelper() : config_path(nullptr) {

	}

	virtual ~CrulerHelper() {
		if (config_path != nullptr) {
			delete config_path;
			config_path = nullptr;
		}
	}

	void set_config_path(std::string& config_path) {
		CrulerHelper::config_path = new std::string(config_path.c_str());
	}

protected:
	std::string* config_path;
};

#endif // !CRULER_HELPER_H
