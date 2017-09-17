#pragma once

extern "C" {
	void cruler_extract_all_with_default_config();
	void cruler_extract_all_from_raw(const char* ext_rule_raw, const char* config_raw);
	void cruler_extract_all(const char* config_path);
}